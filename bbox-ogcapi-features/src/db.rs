use deadpool_postgres::Client;

pub async fn db_query<F>(client: &Client, mut read: F)
where
    F: FnMut(&str),
{
    // http://blog.cleverelephant.ca/2019/03/geojson.html
    let stmt = client
        .prepare(
            &"SELECT rowjsonb_to_geojson(to_jsonb(ne.admin_0_countries.*), 'wkb_geometry') \
        FROM ne.admin_0_countries",
        )
        .await
        .unwrap();

    for row in client.query(&stmt, &[]).await.unwrap() {
        read(row.get::<_, &str>(0));
    }

    // Using RowStream
    // use futures::{StreamExt, TryStreamExt};
    // use std::io;
    // let fut = client.query_raw(&stmt, &[]);
    // let mut items = vec![];
    // Box::pin(async move {
    //     let mut stream = fut
    //         .await
    //         .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e))).unwrap();
    //     while let Some(row) = stream.next().await {
    //         let row = row.map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e))).unwrap();
    //         items.push(row.get::<_, i32>(0));
    //     }
    // });
}
