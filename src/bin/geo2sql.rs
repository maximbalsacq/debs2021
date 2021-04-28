use debs2021::io::load_locations;
use debs2021::gen::challenger::Polygon;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt,BufWriter};

fn make_polygon_string(polys: &[Polygon]) -> String {
    let polys = polys
        .iter()
        .map(|poly| {
            assert_eq!(poly.points.first(), poly.points.last());
            let points = poly.points
                .iter()
                .map(|p| format!("{} {}", p.latitude, p.longitude))
                .collect::<Vec<String>>();
            format!("(({}))", points.join(","))
        })
        .collect::<Vec<String>>();
    format!("multipolygon( {} )", polys.join(","))
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::var("DEBS_DATA_ROOT").expect("DEBS_DATA_ROOT not set!");
    let location = load_locations(&root).await?;   
    let mut out = BufWriter::new(File::create(format!("{}/locations.sql", &root)).await?);
    let tag = "xpSQLe";
    out.write_all("BEGIN;\n".as_bytes()).await?;
    for location in location.locations {
        assert_eq!(location.zipcode.chars().any(|c| !c.is_digit(10)), false);
        let polygon_string = make_polygon_string(&location.polygons);
        let s = format!("INSERT INTO locations VALUES ('{}', ${}${}${}$, ST_GeomFromEWKT('{}'));\n", location.zipcode, tag, location.city, tag, polygon_string);
        out.write_all(s.as_bytes()).await?;
    }
    out.write_all("COMMIT;\n".as_bytes()).await?;
    out.flush().await?;
    Ok(())
}
