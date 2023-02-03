// collection of parameters that may speed up the chromium perf on headless
#[allow(unused)]
fn speed_params() -> Vec<&'static str> {
    let params = ["--disable-canvas-aa", // Disable antialiasing on 2d canvas
    "--disable-2d-canvas-clip-aa", // Disable antialiasing on 2d canvas clips
    "--disable-gl-drawing-for-tests", // BEST OPTION EVER! Disables GL drawing operations which produce pixel output. With this the GL output will not be correct but tests will run faster.
    "--disable-dev-shm-usage", // ???
    "--no-zygote", // wtf does that mean ?
    "--use-gl=swiftshader", // better cpu usage with --use-gl=desktop rather than --use-gl=swiftshader, still needs more testing.
    "--enable-webgl",
    "--hide-scrollbars",
    "--mute-audio",
    "--no-first-run",
    "--disable-infobars",
    "--disable-breakpad",
    "--window-size=1280,1024", // see defaultViewport
    "--no-sandbox", // meh but better resource comsuption
    "--disable-setuid-sandbox",
    "--ignore-certificate-errors",
    "--disable-extensions",
    "--disable-gpu",
    "--no-sandbox"];
    Vec::from(params)
}

#[allow(unused)]
pub fn headless_params(log: bool) -> Vec<String> {
    let binding = std::env::temp_dir();
    let dir = binding.to_str().unwrap();
    let mut params  = speed_params();
    let data_dir = String::from("--user-data-dir=") + dir;
    let mut p = vec!("--headless",  "--remote-debugging-port=9222");
    p.push(data_dir.as_str());
    if cfg!(target_os = "windows") {
        p.push(" --no-sandbox");
        p.push("--disable-gpu");
    }

    if log {
        p.push("--enable-logging --v=0");
    } else {
        p.push(" -disable-logging");
    }
    
    params.append(&mut p);
    params.into_iter().map(|s| {String::from(s)} ).collect()
}

#[allow(unused)]
pub fn system_chrome() -> Option<(String, Vec<String>)> {
    if cfg!(target_os = "unix") {
        return Some((String::from("chromium-browser"), Vec::new()));
    }
    if cfg!(target_os = "macos") {
        return Some((String::from(r#"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"#), Vec::new()));
    }
    if cfg!(target_os = "windows") {
        return Some((
         r#"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"#.to_string(), Vec::new()));
    }
    None
}


#[allow(unused)]
pub fn kill_headless() -> bool {
    let cmd = 
    if cfg!(target_os = "windows") {
        ("powershell.exe", vec!("-command",
         r#""Get-CimInstance -ClassName Win32_Process -Filter 'CommandLine LIKE ''%--headless%'' | %{Stop-Process -Id $_.ProcessId}""#))
    } else {    
        //("pkill", vec!(r#"-f \"(chrome)?(--headless)\""#))
        ("pkill", vec!("-f", r#"Google Chrome.*headless"#))
    };

    let output = std::process::Command::new(cmd.0)
    .args(cmd.1)
    .output();


    match output {
        Ok(out) => {
            println!("kill headless: status: {}", out.status);
            if (!out.status.success()) {
                let cout = std::str::from_utf8(&out.stdout);
                let cerr = std::str::from_utf8(&out.stderr);
                println!("headless: killed: out: {}\n err: {}", cout.unwrap_or("???"), cerr.unwrap_or("???"));
            } else {
                println!("UI killed ok");
            }
            true // here we get handle to spawned UI - not used now as exit is done nicely
        },
        Err(e) => {
            eprintln!("Kill error: {}", e);
            false
        }
    } 
}
