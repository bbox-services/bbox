mod fcgi_process;
mod webserver;

use crate::fcgi_process::{do_request, FcgiProcess};

pub async fn exec_wms_fcgi(fcgi_app: &str) -> std::io::Result<()> {
    let mut process = FcgiProcess::spawn(fcgi_app).await?;
    process.wait_until_ready();
    let mut fcgi_client = process.handler().fcgi_client()?;
    do_request(
        &mut fcgi_client,
        "map=test/helloworld.qgs&SERVICE=WMS&REQUEST=GetCapabilities",
    )?;
    // do_request(&mut fcgi_client, "map=test/helloworld.qgs&ERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&BBOX=-67.593,-176.248,83.621,182.893&CRS=EPSG:4326&WIDTH=515&HEIGHT=217&LAYERS=Country,Hello&STYLES=,&FORMAT=image/png;%20mode%3D8bit&DPI=96&TRANSPARENT=TRUE").await?;
    process.dump_stderr().await?;
    Ok(())
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    // futures_lite::future::block_on(exec_wms_fcgi("/usr/lib/cgi-bin/qgis_mapserv.fcgi")).unwrap();
    // futures_lite::future::block_on(exec_wms_fcgi("/usr/lib/cgi-bin/mapserv")).unwrap();
    webserver::webserver().unwrap();
}
