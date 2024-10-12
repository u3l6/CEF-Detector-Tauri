use crate::{CefResult, CefSize};
use base64::prelude::*;
use encoding_rs::GBK;
use size::Size;
use std::{collections::HashSet, error::Error, os::windows::process::CommandExt, path::Path};
use tauri::ipc::Channel;

fn decode(raw: Vec<u8>) -> Result<String, Box<dyn Error + Send + Sync>> {
    let (decoded, _, _) = GBK.decode(&raw);
    Ok(decoded.into_owned())
}

async fn search(args: &[&str]) -> Result<String, Box<dyn Error + Send + Sync>> {
    let output = tokio::process::Command::new("es.exe")
        .args(args)
        .creation_flags(0x08000000)
        .output()
        .await?;
    if !output.stderr.is_empty() {
        return Err(decode(output.stderr)?.into());
    }
    Ok(decode(output.stdout)?)
}

async fn get_folder(list: String) -> Result<HashSet<String>, Box<dyn Error + Send + Sync>> {
    let paths: HashSet<String> = list
        .lines()
        .filter(|path| {
            !path.is_empty()
                && !path.contains(r"\$Recycle.Bin\")
                && !path.contains(r"\$RECYCLE.BIN\")
                && !path.contains(r"\OneDrive\")
                && !path.contains(r"C:\Windows")
        })
        .filter_map(|path| {
            let parent_path = Path::new(path).parent();
            let parent_path = parent_path.and_then(|p| p.to_str());
            parent_path.map(|s| s.to_owned())
        })
        .collect();
    Ok(paths)
}

async fn get_path_icon(path: &String) -> Result<(String, String), Box<dyn Error + Send + Sync>> {
    let output = search(&["-p", &path, "-r", r"\.exe$"]).await?;
    let base64: serde_json::Value = serde_json::from_str(include_str!(r"..\base64.json"))?;
    let base64 = &base64["icons"];
    for exe in output.lines() {
        let raw_icon = systemicons::get_icon(exe, 32).unwrap();
        let base64_icon = BASE64_STANDARD.encode(&raw_icon);
        if base64
            .as_array()
            .map_or(false, |icons| icons.iter().all(|icon| icon != &base64_icon))
        {
            return Ok((exe.to_owned(), base64_icon));
        }
    }
    let parent = Path::new(path)
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
    Ok((parent, "".to_owned()))
}

async fn get_size(path: &String) -> Result<Size, Box<dyn Error + Send + Sync>> {
    let output = search(&["-size", "-no-digit-grouping", "/a-d", "-p", &path]).await?;
    let mut total_size = Size::from_bytes(0);
    for line in output.lines() {
        if let Some(index) = line.trim().find(" ") {
            let size = line.trim()[..index].to_owned();
            total_size += Size::from_bytes(size.parse::<u64>()?);
        }
    }
    Ok(total_size)
}

pub async fn command(channel: Channel<CefResult>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let (search_result_pak, search_result_libcef, search_result_node) = tokio::join!(
        search(&["-r", r"(_100_)(.+?)(\.pak$)"]),
        search(&["-r", "^libcef.dll$"]),
        search(&["-r", r"^node(.*?)\.dll$"])
    );
    let search_result = format!(
        "{}{}{}",
        search_result_pak?, search_result_libcef?, search_result_node?
    );
    let paths = get_folder(search_result).await?;
    let count = paths.len();
    channel.send(CefResult::Count(count))?;
    #[cfg(debug_assertions)]
    println!("{:?}", CefResult::Count(count));
    let mut total_size = Size::from_bytes(0);
    let mut cef_result: Vec<CefResult> = vec![];
    for input_path in paths {
        let channel_clone = channel.clone();
        let task = tokio::spawn(async move {
            let (path_icon, size) = tokio::join!(get_path_icon(&input_path), get_size(&input_path));
            let ((mut path, mut icon), size) = (path_icon?, size?);
            for _ in 0..10 {
                if path.ends_with(".exe") {
                    break;
                } else {
                    (path, icon) = get_path_icon(&path).await?;
                }
            }
            if icon.is_empty() {
                path = input_path;
                let base64: serde_json::Value =
                    serde_json::from_str(include_str!(r"..\base64.json"))?;
                let base64 = &base64["icons"];
                icon = base64[3].to_string().replace("\"", "");
            }
            let name = Path::new(&path)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();
            let result = CefResult::Path {
                name: name.clone(),
                path: path.clone(),
                icon: icon.clone(),
                size: CefSize(size.clone()),
            };
            channel_clone.send(result.clone())?;
            #[cfg(debug_assertions)]
            println!("{:?}", result);
            Ok((result, size)) as Result<(CefResult, Size), Box<dyn Error + Send + Sync>>
        });
        let (result, size) = tokio::join!(task).0??;
        total_size += size;
        cef_result.push(result);
        channel.send(CefResult::TotalSize(CefSize(total_size)))?;
        #[cfg(debug_assertions)]
        println!("{:?}", CefResult::TotalSize(CefSize(total_size)));
    }
    cef_result.sort_by(|a, b| match (a, b) {
        (CefResult::Path { size: size_a, .. }, CefResult::Path { size: size_b, .. }) => {
            let size_a = *size_a.clone();
            let size_b = *size_b.clone();
            size_b.partial_cmp(&size_a).unwrap()
        }
        _ => std::cmp::Ordering::Equal,
    });
    channel.send(CefResult::Sign)?;
    for cef_result in cef_result.iter() {
        channel.send(cef_result.clone())?;
    }
    println!("{:#?}", cef_result);
    Ok(())
}

#[tauri::command]
pub fn open_folder(path: String) -> Result<(), String> {
    let output = std::process::Command::new("explorer")
        .arg("/select,")
        .arg(path)
        .creation_flags(0x08000000)
        .output()
        .unwrap();
    if !output.stderr.is_empty() {
        return Err(decode(output.stderr).unwrap());
    }
    Ok(())
}
