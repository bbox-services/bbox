use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename = "WMS_Capabilities")]
pub struct WmsCapabilities {
    pub version: String,
    #[serde(rename = "Service")]
    pub service: Service,
    #[serde(rename = "Capability")]
    pub capability: Capability,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Service {
    pub name: String,
    pub title: String,
    pub keyword_list: Option<KeywordList>,
    //...
}

#[derive(Deserialize, Debug)]
pub struct KeywordList {
    #[serde(rename = "Keyword", default)]
    pub keywords: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Capability {
    // Request
    // Exception
    // UserDefinedSymbolization
    #[serde(rename = "Layer", default)]
    pub layers: Vec<Layer>,
    // QGIS extended capabilities
    // WFSLayers
    // LayerDrawingOrder
}

#[derive(Deserialize, Debug)]
pub struct Layer {
    pub queryable: Option<bool>,
    pub opaque: Option<bool>,
    pub cascaded: Option<bool>,
    #[serde(rename = "Name")]
    pub name: Option<String>,
    #[serde(rename = "Title")]
    pub title: Option<String>,
    #[serde(rename = "Abstract")]
    pub abstract_: Option<String>,

    #[serde(rename = "CRS", default)]
    pub crs: Vec<String>,
    #[serde(rename = "EX_GeographicBoundingBox")]
    pub ex_geographic_bounding_box: Option<ExGeographicBoundingBox>,
    #[serde(rename = "BoundingBox", default)]
    pub bounding_box: Vec<BoundingBox>,
    // pub Style
    #[serde(rename = "MinScaleDenominator")]
    pub min_scale_denominator: Option<f32>,
    #[serde(rename = "MaxScaleDenominator")]
    pub max_scale_denominator: Option<f32>,

    #[serde(rename = "Attribution")]
    pub attribution: Option<Attribution>,
    #[serde(rename = "DataURL")]
    pub data_url: Option<MetadataUrl>,
    #[serde(rename = "MetadataURL")]
    pub metadata_url: Option<MetadataUrl>,
    #[serde(rename = "Layer", default)]
    pub layers: Vec<Layer>,

    // QGIS extended capabilities
    pub visible: Option<bool>,
    pub opacity: Option<f32>,
    pub expanded: Option<bool>,
    #[serde(rename = "mutuallyExclusive")]
    pub mutually_exclusive: Option<bool>,
    #[serde(rename = "displayField")]
    pub display_field: Option<String>,
    #[serde(rename = "geometryType")]
    pub geometry_type: Option<String>,
    // pub TreeName
    // pub PrimaryKey
    // pub Attributes
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExGeographicBoundingBox {
    pub west_bound_longitude: f64,
    pub east_bound_longitude: f64,
    pub south_bound_latitude: f64,
    pub north_bound_latitude: f64,
}

#[derive(Deserialize, Debug)]
pub struct BoundingBox {
    #[serde(rename = "CRS")]
    pub crs: String,
    pub minx: f64,
    pub miny: f64,
    pub maxx: f64,
    pub maxy: f64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Attribution {
    pub title: String,
    pub online_resource: OnlineResource,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MetadataUrl {
    pub online_resource: OnlineResource,
    // type
    // Format
}

#[derive(Deserialize, Debug)]
pub struct OnlineResource {
    pub href: String,
    // xlink
    // type
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_xml_rs::from_reader;

    #[test]
    fn umn() {
        let s = r##"
<?xml version='1.0' encoding="UTF-8" standalone="no" ?>
<WMS_Capabilities version="1.3.0"  xmlns="http://www.opengis.net/wms"   xmlns:sld="http://www.opengis.net/sld"   xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"   xmlns:ms="http://mapserver.gis.umn.edu/mapserver"   xsi:schemaLocation="http://www.opengis.net/wms http://schemas.opengis.net/wms/1.3.0/capabilities_1_3_0.xsd  http://www.opengis.net/sld http://schemas.opengis.net/sld/1.1.0/sld_capabilities.xsd  http://mapserver.gis.umn.edu/mapserver http://localhost/cgi-bin/mapserv?service=WMS&amp;version=1.3.0&amp;request=GetSchemaExtension">

<!-- MapServer version 7.4.3 OUTPUT=PNG OUTPUT=JPEG OUTPUT=KML SUPPORTS=PROJ SUPPORTS=AGG SUPPORTS=FREETYPE SUPPORTS=CAIRO SUPPORTS=SVG_SYMBOLS SUPPORTS=RSVG SUPPORTS=ICONV SUPPORTS=FRIBIDI SUPPORTS=WMS_SERVER SUPPORTS=WMS_CLIENT SUPPORTS=WFS_SERVER SUPPORTS=WFS_CLIENT SUPPORTS=WCS_SERVER SUPPORTS=SOS_SERVER SUPPORTS=FASTCGI SUPPORTS=THREADS SUPPORTS=GEOS SUPPORTS=PBF INPUT=JPEG INPUT=POSTGIS INPUT=OGR INPUT=GDAL INPUT=SHAPEFILE -->

<Service>
  <Name>WMS</Name>
  <Title>Test mapfile for MVT development.</Title>
  <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://localhost/cgi-bin/mapserv?"/>
  <ContactInformation>
  </ContactInformation>
  <MaxWidth>4096</MaxWidth>
  <MaxHeight>4096</MaxHeight>
</Service>

<Capability>
  <Request>
    <GetCapabilities>
      <Format>text/xml</Format>
      <DCPType>
        <HTTP>
          <Get><OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://localhost/cgi-bin/mapserv?"/></Get>
          <Post><OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://localhost/cgi-bin/mapserv?"/></Post>
        </HTTP>
      </DCPType>
    </GetCapabilities>
    <GetMap>
      <Format>image/png; mode=8bit</Format>
      <Format>application/x-protobuf</Format>
      <Format>image/png</Format>
      <Format>image/jpeg</Format>
      <Format>image/vnd.jpeg-png</Format>
      <Format>image/vnd.jpeg-png8</Format>
      <Format>application/x-pdf</Format>
      <Format>image/svg+xml</Format>
      <Format>image/tiff</Format>
      <Format>application/vnd.google-earth.kml+xml</Format>
      <Format>application/vnd.google-earth.kmz</Format>
      <Format>application/json</Format>
      <DCPType>
        <HTTP>
          <Get><OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://localhost/cgi-bin/mapserv?"/></Get>
          <Post><OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://localhost/cgi-bin/mapserv?"/></Post>
        </HTTP>
      </DCPType>
    </GetMap>
    <GetFeatureInfo>
      <Format>text/plain</Format>
      <Format>application/vnd.ogc.gml</Format>
      <DCPType>
        <HTTP>
          <Get><OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://localhost/cgi-bin/mapserv?"/></Get>
          <Post><OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://localhost/cgi-bin/mapserv?"/></Post>
        </HTTP>
      </DCPType>
    </GetFeatureInfo>
    <sld:DescribeLayer>
      <Format>text/xml</Format>
      <DCPType>
        <HTTP>
          <Get><OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://localhost/cgi-bin/mapserv?"/></Get>
          <Post><OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://localhost/cgi-bin/mapserv?"/></Post>
        </HTTP>
      </DCPType>
    </sld:DescribeLayer>
    <sld:GetLegendGraphic>
      <Format>image/png; mode=8bit</Format>
      <Format>image/png</Format>
      <Format>image/jpeg</Format>
      <Format>image/vnd.jpeg-png</Format>
      <Format>image/vnd.jpeg-png8</Format>
      <DCPType>
        <HTTP>
          <Get><OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://localhost/cgi-bin/mapserv?"/></Get>
          <Post><OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://localhost/cgi-bin/mapserv?"/></Post>
        </HTTP>
      </DCPType>
    </sld:GetLegendGraphic>
    <ms:GetStyles>
      <Format>text/xml</Format>
      <DCPType>
        <HTTP>
          <Get><OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://localhost/cgi-bin/mapserv?"/></Get>
          <Post><OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://localhost/cgi-bin/mapserv?"/></Post>
        </HTTP>
      </DCPType>
    </ms:GetStyles>
  </Request>
  <Exception>
    <Format>XML</Format>
    <Format>INIMAGE</Format>
    <Format>BLANK</Format>
  </Exception>
  <sld:UserDefinedSymbolization SupportSLD="1" UserLayer="0" UserStyle="1" RemoteWFS="0" InlineFeature="0" RemoteWCS="0"/>
  <Layer>
    <Name>ne_countries</Name>
    <Title>Test mapfile for MVT development.</Title>
    <Abstract>ne_countries</Abstract>
    <CRS>epsg:3857</CRS>
    <CRS>epsg:4326</CRS>
    <EX_GeographicBoundingBox>
        <westBoundLongitude>-8.98315e-06</westBoundLongitude>
        <eastBoundLongitude>-8.98315e-06</eastBoundLongitude>
        <southBoundLatitude>-8.98315e-06</southBoundLatitude>
        <northBoundLatitude>-8.98315e-06</northBoundLatitude>
    </EX_GeographicBoundingBox>
    <BoundingBox CRS="EPSG:3857"
                minx="-1" miny="-1" maxx="-1" maxy="-1" />
    <Layer queryable="0" opaque="0" cascaded="0">
        <Name>country</Name>
        <Title>country</Title>
        <CRS>epsg:4326</CRS>
        <CRS>epsg:3857</CRS>
        <CRS>epsg:900913</CRS>
        <EX_GeographicBoundingBox>
            <westBoundLongitude>-180</westBoundLongitude>
            <eastBoundLongitude>180</eastBoundLongitude>
            <southBoundLatitude>-89.5014</southBoundLatitude>
            <northBoundLatitude>83.6341</northBoundLatitude>
        </EX_GeographicBoundingBox>
        <BoundingBox CRS="epsg:4326"
                    minx="-89.5014" miny="-180" maxx="83.6341" maxy="180" />
        <MetadataURL type="TC211">
          <Format>text/xml</Format>
          <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:type="simple" xlink:href="http://localhost/cgi-bin/mapserv?request=GetMetadata&amp;layer=country"/>
        </MetadataURL>
    </Layer>
  </Layer>
</Capability>
</WMS_Capabilities>
    "##;
        let cap: WmsCapabilities = from_reader(s.as_bytes()).unwrap();
        println!("{:#?}", cap);

        assert_eq!(cap.version, "1.3.0");
        assert_eq!(cap.service.name, "WMS");
        assert_eq!(cap.service.title, "Test mapfile for MVT development.");

        assert_eq!(
            cap.capability.layers[0].name,
            Some("ne_countries".to_string())
        );
        assert_eq!(
            cap.capability.layers[0].title,
            Some("Test mapfile for MVT development.".to_string())
        );
        assert_eq!(
            cap.capability.layers[0].abstract_,
            Some("ne_countries".to_string())
        );
        assert_eq!(cap.capability.layers[0].crs, vec!["epsg:3857", "epsg:4326"]);

        let layer = &cap.capability.layers[0].layers[0];
        assert_eq!(layer.name, Some("country".to_string()));
        assert_eq!(
            layer.metadata_url.as_ref().unwrap().online_resource.href,
            "http://localhost/cgi-bin/mapserv?request=GetMetadata&layer=country"
        );
        assert_eq!(layer.crs, vec!["epsg:4326", "epsg:3857", "epsg:900913"]);
    }

    #[test]
    fn qgis() {
        let s = r##"
<?xml version="1.0" encoding="utf-8"?>
<WMS_Capabilities xmlns:sld="http://www.opengis.net/sld" version="1.3.0" xmlns="http://www.opengis.net/wms" xmlns:qgs="http://www.qgis.org/wms" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.opengis.net/wms http://schemas.opengis.net/wms/1.3.0/capabilities_1_3_0.xsd http://www.opengis.net/sld http://schemas.opengis.net/sld/1.1.0/sld_capabilities.xsd http://www.qgis.org/wms http://127.0.0.1:8080/wms/qgs/ne?SERVICE=WMS&amp;REQUEST=GetSchemaExtension">
 <Service>
  <Name>WMS</Name>
  <Title>untitled</Title>
  <KeywordList>
   <Keyword vocabulary="ISO">infoMapAccessService</Keyword>
  </KeywordList>
  <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:type="simple" xlink:href="http://127.0.0.1:8080/wms/qgs/ne"/>
  <Fees>conditions unknown</Fees>
  <AccessConstraints>None</AccessConstraints>
 </Service>
 <Capability>
  <Request>
   <GetCapabilities>
    <Format>text/xml</Format>
    <DCPType>
     <HTTP>
      <Get>
       <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:type="simple" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?"/>
      </Get>
     </HTTP>
    </DCPType>
   </GetCapabilities>
   <GetMap>
    <Format>image/jpeg</Format>
    <Format>image/png</Format>
    <Format>image/png; mode=16bit</Format>
    <Format>image/png; mode=8bit</Format>
    <Format>image/png; mode=1bit</Format>
    <Format>application/dxf</Format>
    <DCPType>
     <HTTP>
      <Get>
       <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:type="simple" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?"/>
      </Get>
     </HTTP>
    </DCPType>
   </GetMap>
   <GetFeatureInfo>
    <Format>text/plain</Format>
    <Format>text/html</Format>
    <Format>text/xml</Format>
    <Format>application/vnd.ogc.gml</Format>
    <Format>application/vnd.ogc.gml/3.1.1</Format>
    <Format>application/json</Format>
    <Format>application/geo+json</Format>
    <DCPType>
     <HTTP>
      <Get>
       <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:type="simple" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?"/>
      </Get>
     </HTTP>
    </DCPType>
   </GetFeatureInfo>
   <sld:GetLegendGraphic>
    <Format>image/jpeg</Format>
    <Format>image/png</Format>
    <Format>application/json</Format>
    <DCPType>
     <HTTP>
      <Get>
       <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:type="simple" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?"/>
      </Get>
     </HTTP>
    </DCPType>
   </sld:GetLegendGraphic>
   <sld:DescribeLayer>
    <Format>text/xml</Format>
    <DCPType>
     <HTTP>
      <Get>
       <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:type="simple" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?"/>
      </Get>
     </HTTP>
    </DCPType>
   </sld:DescribeLayer>
   <qgs:GetStyles>
    <Format>text/xml</Format>
    <DCPType>
     <HTTP>
      <Get>
       <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:type="simple" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?"/>
      </Get>
     </HTTP>
    </DCPType>
   </qgs:GetStyles>
  </Request>
  <Exception>
   <Format>XML</Format>
  </Exception>
  <sld:UserDefinedSymbolization RemoteWFS="0" SupportSLD="1" UserStyle="1" InlineFeature="0" RemoteWCS="0" UserLayer="0"/>
  <Layer queryable="1">
   <KeywordList>
    <Keyword vocabulary="ISO">infoMapAccessService</Keyword>
   </KeywordList>
   <CRS>CRS:84</CRS>
   <CRS>EPSG:3857</CRS>
   <CRS>EPSG:4326</CRS>
   <EX_GeographicBoundingBox>
    <westBoundLongitude>-179.999926</westBoundLongitude>
    <eastBoundLongitude>179.999927</eastBoundLongitude>
    <southBoundLatitude>-89.999996</southBoundLatitude>
    <northBoundLatitude>89.999996</northBoundLatitude>
   </EX_GeographicBoundingBox>
   <BoundingBox miny="-179.999926" maxy="179.999927" minx="-89.999996" maxx="89.999996" CRS="EPSG:4326"/>
   <BoundingBox miny="-109516377.571" maxy="109516377.539" minx="-20037500.106" maxx="20037500.106" CRS="EPSG:3857"/>
   <Layer queryable="1">
    <Name>country-name</Name>
    <Title>country-name</Title>
    <CRS>CRS:84</CRS>
    <CRS>EPSG:3857</CRS>
    <CRS>EPSG:4326</CRS>
    <EX_GeographicBoundingBox>
     <westBoundLongitude>-177.228623</westBoundLongitude>
     <eastBoundLongitude>178.519502</eastBoundLongitude>
     <southBoundLatitude>-80.516517</southBoundLatitude>
     <northBoundLatitude>73.348998</northBoundLatitude>
    </EX_GeographicBoundingBox>
    <BoundingBox miny="-177.228623" maxy="178.519502" minx="-80.516517" maxx="73.348998" CRS="EPSG:4326"/>
    <BoundingBox miny="-15878600" maxy="12257700" minx="-19729000" maxx="19872700" CRS="EPSG:3857"/>
    <Style>
     <Name>default</Name>
     <Title>default</Title>
     <LegendURL>
      <Format>image/png</Format>
      <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:type="simple" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?&amp;SERVICE=WMS&amp;VERSION=1.3.0&amp;REQUEST=GetLegendGraphic&amp;LAYER=country-name&amp;FORMAT=image/png&amp;STYLE=default&amp;SLD_VERSION=1.1.0"/>
     </LegendURL>
    </Style>
   </Layer>
   <Layer queryable="1">
    <Name>state</Name>
    <Title>state</Title>
    <CRS>CRS:84</CRS>
    <CRS>EPSG:3857</CRS>
    <CRS>EPSG:4326</CRS>
    <EX_GeographicBoundingBox>
     <westBoundLongitude>-139.060207</westBoundLongitude>
     <eastBoundLongitude>153.506568</eastBoundLongitude>
     <southBoundLatitude>-39.201702</southBoundLatitude>
     <northBoundLatitude>78.686917</northBoundLatitude>
    </EX_GeographicBoundingBox>
    <BoundingBox miny="-139.060207" maxy="153.506568" minx="-39.201702" maxx="78.686917" CRS="EPSG:4326"/>
    <BoundingBox miny="-4750604.908" maxy="14747260.316" minx="-15480111.391" maxx="17088272.979" CRS="EPSG:3857"/>
    <Style>
     <Name>default</Name>
     <Title>default</Title>
     <LegendURL>
      <Format>image/png</Format>
      <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:type="simple" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?&amp;SERVICE=WMS&amp;VERSION=1.3.0&amp;REQUEST=GetLegendGraphic&amp;LAYER=state&amp;FORMAT=image/png&amp;STYLE=default&amp;SLD_VERSION=1.1.0"/>
     </LegendURL>
    </Style>
   </Layer>
   <Layer queryable="1">
    <Name>country</Name>
    <Title>country</Title>
    <CRS>CRS:84</CRS>
    <CRS>EPSG:3857</CRS>
    <CRS>EPSG:4326</CRS>
    <EX_GeographicBoundingBox>
     <westBoundLongitude>-179.999926</westBoundLongitude>
     <eastBoundLongitude>179.999926</eastBoundLongitude>
     <southBoundLatitude>-89.501388</southBoundLatitude>
     <northBoundLatitude>83.634081</northBoundLatitude>
    </EX_GeographicBoundingBox>
    <BoundingBox miny="-179.999926" maxy="179.999926" minx="-89.501388" maxx="83.634081" CRS="EPSG:4326"/>
    <BoundingBox miny="-34679800" maxy="18428900" minx="-20037500" maxx="20037500" CRS="EPSG:3857"/>
    <Style>
     <Name>default</Name>
     <Title>default</Title>
     <LegendURL>
      <Format>image/png</Format>
      <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:type="simple" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?&amp;SERVICE=WMS&amp;VERSION=1.3.0&amp;REQUEST=GetLegendGraphic&amp;LAYER=country&amp;FORMAT=image/png&amp;STYLE=default&amp;SLD_VERSION=1.1.0"/>
     </LegendURL>
    </Style>
   </Layer>
   <Layer queryable="1">
    <Name>geo-lines</Name>
    <Title>geo-lines</Title>
    <CRS>CRS:84</CRS>
    <CRS>EPSG:3857</CRS>
    <CRS>EPSG:4326</CRS>
    <EX_GeographicBoundingBox>
     <westBoundLongitude>-179.999926</westBoundLongitude>
     <eastBoundLongitude>179.999926</eastBoundLongitude>
     <southBoundLatitude>-89.999996</southBoundLatitude>
     <northBoundLatitude>89.999996</northBoundLatitude>
    </EX_GeographicBoundingBox>
    <BoundingBox miny="-179.999926" maxy="179.999926" minx="-89.999996" maxx="89.999996" CRS="EPSG:4326"/>
    <BoundingBox miny="-109516377.571" maxy="109516377.539" minx="-20037500.106" maxx="20037500.106" CRS="EPSG:3857"/>
    <Layer queryable="1">
     <Name>ne_10m_geographic_lines</Name>
     <Title>ne_10m_geographic_lines</Title>
     <CRS>CRS:84</CRS>
     <CRS>EPSG:3857</CRS>
     <CRS>EPSG:4326</CRS>
     <EX_GeographicBoundingBox>
      <westBoundLongitude>-179.999926</westBoundLongitude>
      <eastBoundLongitude>179.999926</eastBoundLongitude>
      <southBoundLatitude>-89.999996</southBoundLatitude>
      <northBoundLatitude>89.999996</northBoundLatitude>
     </EX_GeographicBoundingBox>
     <BoundingBox miny="-179.999926" maxy="179.999926" minx="-89.999996" maxx="89.999996" CRS="EPSG:4326"/>
     <BoundingBox miny="-108777000" maxy="108777000" minx="-20037500" maxx="20037500" CRS="EPSG:3857"/>
     <Style>
      <Name>default</Name>
      <Title>default</Title>
      <LegendURL>
       <Format>image/png</Format>
       <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:type="simple" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?&amp;SERVICE=WMS&amp;VERSION=1.3.0&amp;REQUEST=GetLegendGraphic&amp;LAYER=ne_10m_geographic_lines&amp;FORMAT=image/png&amp;STYLE=default&amp;SLD_VERSION=1.1.0"/>
      </LegendURL>
     </Style>
     <MinScaleDenominator>0</MinScaleDenominator>
     <MaxScaleDenominator>5e+06</MaxScaleDenominator>
    </Layer>
    <Layer queryable="1">
     <Name>ne_50m_geographic_lines</Name>
     <Title>ne_50m_geographic_lines</Title>
     <CRS>CRS:84</CRS>
     <CRS>EPSG:3857</CRS>
     <CRS>EPSG:4326</CRS>
     <EX_GeographicBoundingBox>
      <westBoundLongitude>-179.999926</westBoundLongitude>
      <eastBoundLongitude>179.999926</eastBoundLongitude>
      <southBoundLatitude>-89.999996</southBoundLatitude>
      <northBoundLatitude>89.999996</northBoundLatitude>
     </EX_GeographicBoundingBox>
     <BoundingBox miny="-179.999926" maxy="179.999926" minx="-89.999996" maxx="89.999996" CRS="EPSG:4326"/>
     <BoundingBox miny="-108777000" maxy="108777000" minx="-20037500" maxx="20037500" CRS="EPSG:3857"/>
     <Style>
      <Name>default</Name>
      <Title>default</Title>
      <LegendURL>
       <Format>image/png</Format>
       <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:type="simple" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?&amp;SERVICE=WMS&amp;VERSION=1.3.0&amp;REQUEST=GetLegendGraphic&amp;LAYER=ne_50m_geographic_lines&amp;FORMAT=image/png&amp;STYLE=default&amp;SLD_VERSION=1.1.0"/>
      </LegendURL>
     </Style>
     <MinScaleDenominator>5e+06</MinScaleDenominator>
     <MaxScaleDenominator>1e+08</MaxScaleDenominator>
    </Layer>
   </Layer>
  </Layer>
 </Capability>
</WMS_Capabilities>
    "##;
        let cap: WmsCapabilities = from_reader(s.as_bytes()).unwrap();
        println!("{:#?}", cap);

        assert_eq!(cap.version, "1.3.0");
        assert_eq!(cap.service.name, "WMS");
        assert_eq!(cap.service.title, "untitled");
        assert_eq!(
            cap.service.keyword_list.as_ref().unwrap().keywords,
            vec!["infoMapAccessService".to_string()]
        );

        assert_eq!(cap.capability.layers[0].name, None);
        assert_eq!(cap.capability.layers[0].title, None);
        assert_eq!(cap.capability.layers[0].queryable, Some(true));
        assert_eq!(cap.capability.layers[0].abstract_, None);
        assert_eq!(
            cap.capability.layers[0].crs,
            vec!["CRS:84", "EPSG:3857", "EPSG:4326"]
        );
        let bbox = cap.capability.layers[0]
            .ex_geographic_bounding_box
            .as_ref()
            .unwrap();
        assert_eq!(bbox.east_bound_longitude, 179.999927);
        assert_eq!(bbox.south_bound_latitude, -89.999996);
        assert_eq!(bbox.west_bound_longitude, -179.999926);
        assert_eq!(bbox.north_bound_latitude, 89.999996);

        assert_eq!(
            cap.capability.layers[0].layers[0].name,
            Some("country-name".to_string())
        );
        assert_eq!(
            cap.capability.layers[0].layers[0].title,
            Some("country-name".to_string())
        );
        assert_eq!(
            cap.capability.layers[0].layers[0].crs,
            vec!["CRS:84", "EPSG:3857", "EPSG:4326"]
        );

        assert_eq!(
            cap.capability.layers[0].layers[3].layers[0].name,
            Some("ne_10m_geographic_lines".to_string())
        );
        assert_eq!(
            cap.capability.layers[0].layers[3].layers[0].min_scale_denominator,
            Some(0.0)
        );
        assert_eq!(
            cap.capability.layers[0].layers[3].layers[0].max_scale_denominator,
            Some(5000000.0)
        );
    }

    #[test]
    fn qgis_getprojectsettings() {
        let s = r##"
<?xml version="1.0" encoding="utf-8"?>
<WMS_Capabilities xsi:schemaLocation="http://www.opengis.net/wms http://schemas.opengis.net/wms/1.3.0/capabilities_1_3_0.xsd http://www.opengis.net/sld http://schemas.opengis.net/sld/1.1.0/sld_capabilities.xsd http://www.qgis.org/wms http://127.0.0.1:8080/wms/qgs/ne?SERVICE=WMS&amp;REQUEST=GetSchemaExtension" xmlns="http://www.opengis.net/wms" xmlns:sld="http://www.opengis.net/sld" version="1.3.0" xmlns:qgs="http://www.qgis.org/wms" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
 <Service>
  <Name>WMS</Name>
  <Title>untitled</Title>
  <KeywordList>
   <Keyword vocabulary="ISO">infoMapAccessService</Keyword>
  </KeywordList>
  <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://127.0.0.1:8080/wms/qgs/ne" xlink:type="simple"/>
  <Fees>conditions unknown</Fees>
  <AccessConstraints>None</AccessConstraints>
 </Service>
 <Capability>
  <Request>
   <GetCapabilities>
    <Format>text/xml</Format>
    <DCPType>
     <HTTP>
      <Get>
       <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?" xlink:type="simple"/>
      </Get>
     </HTTP>
    </DCPType>
   </GetCapabilities>
   <GetMap>
    <Format>image/jpeg</Format>
    <Format>image/png</Format>
    <Format>image/png; mode=16bit</Format>
    <Format>image/png; mode=8bit</Format>
    <Format>image/png; mode=1bit</Format>
    <Format>application/dxf</Format>
    <DCPType>
     <HTTP>
      <Get>
       <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?" xlink:type="simple"/>
      </Get>
     </HTTP>
    </DCPType>
   </GetMap>
   <GetFeatureInfo>
    <Format>text/plain</Format>
    <Format>text/html</Format>
    <Format>text/xml</Format>
    <Format>application/vnd.ogc.gml</Format>
    <Format>application/vnd.ogc.gml/3.1.1</Format>
    <Format>application/json</Format>
    <Format>application/geo+json</Format>
    <DCPType>
     <HTTP>
      <Get>
       <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?" xlink:type="simple"/>
      </Get>
     </HTTP>
    </DCPType>
   </GetFeatureInfo>
   <sld:GetLegendGraphic>
    <Format>image/jpeg</Format>
    <Format>image/png</Format>
    <Format>application/json</Format>
    <DCPType>
     <HTTP>
      <Get>
       <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?" xlink:type="simple"/>
      </Get>
     </HTTP>
    </DCPType>
   </sld:GetLegendGraphic>
   <sld:DescribeLayer>
    <Format>text/xml</Format>
    <DCPType>
     <HTTP>
      <Get>
       <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?" xlink:type="simple"/>
      </Get>
     </HTTP>
    </DCPType>
   </sld:DescribeLayer>
   <qgs:GetStyles>
    <Format>text/xml</Format>
    <DCPType>
     <HTTP>
      <Get>
       <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?" xlink:type="simple"/>
      </Get>
     </HTTP>
    </DCPType>
   </qgs:GetStyles>
   <GetPrint>
    <Format>svg</Format>
    <Format>png</Format>
    <Format>pdf</Format>
    <DCPType>
     <HTTP>
      <Get>
       <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?" xlink:type="simple"/>
      </Get>
     </HTTP>
    </DCPType>
   </GetPrint>
  </Request>
  <Exception>
   <Format>XML</Format>
  </Exception>
  <sld:UserDefinedSymbolization RemoteWFS="0" UserStyle="1" SupportSLD="1" UserLayer="0" RemoteWCS="0" InlineFeature="0"/>
  <WFSLayers>
   <WFSLayer name="country"/>
   <WFSLayer name="state"/>
  </WFSLayers>
  <Layer queryable="1">
   <KeywordList>
    <Keyword vocabulary="ISO">infoMapAccessService</Keyword>
   </KeywordList>
   <CRS>CRS:84</CRS>
   <CRS>EPSG:3857</CRS>
   <CRS>EPSG:4326</CRS>
   <EX_GeographicBoundingBox>
    <westBoundLongitude>-179.999926</westBoundLongitude>
    <eastBoundLongitude>179.999927</eastBoundLongitude>
    <southBoundLatitude>-89.999996</southBoundLatitude>
    <northBoundLatitude>89.999996</northBoundLatitude>
   </EX_GeographicBoundingBox>
   <BoundingBox maxy="179.999927" miny="-179.999926" CRS="EPSG:4326" maxx="89.999996" minx="-89.999996"/>
   <BoundingBox maxy="109516377.539" miny="-109516377.571" CRS="EPSG:3857" maxx="20037500.106" minx="-20037500.106"/>
   <TreeName></TreeName>
   <Layer geometryType="Point" queryable="1" opacity="1" visible="1" expanded="1" displayField="z_name">
    <Name>ne</Name>
    <Title>Natural Earth</Title>
    <KeywordList>
     <Keyword>countries</Keyword>
     <Keyword>political</Keyword>
    </KeywordList>
    <CRS>CRS:84</CRS>
    <CRS>EPSG:3857</CRS>
    <CRS>EPSG:4326</CRS>
    <EX_GeographicBoundingBox>
     <westBoundLongitude>-177.228623</westBoundLongitude>
     <eastBoundLongitude>178.519502</eastBoundLongitude>
     <southBoundLatitude>-80.516517</southBoundLatitude>
     <northBoundLatitude>73.348998</northBoundLatitude>
    </EX_GeographicBoundingBox>
    <BoundingBox maxy="178.519502" miny="-177.228623" CRS="EPSG:4326" maxx="73.348998" minx="-80.516517"/>
    <BoundingBox maxy="12257700" miny="-15878600" CRS="EPSG:3857" maxx="19872700" minx="-19729000"/>
    <Style>
     <Name>default</Name>
     <Title>default</Title>
     <LegendURL>
      <Format>image/png</Format>
      <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?&amp;SERVICE=WMS&amp;VERSION=1.3.0&amp;REQUEST=GetLegendGraphic&amp;LAYER=ne&amp;FORMAT=image/png&amp;STYLE=default&amp;SLD_VERSION=1.1.0" xlink:type="simple"/>
     </LegendURL>
    </Style>
    <DataURL>
     <Format>text/plain</Format>
     <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="https://www.naturalearthdata.com/" xlink:type="simple"/>
    </DataURL>
    <Attribution>
     <Title>Natural Earth</Title>
     <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="https://www.naturalearthdata.com/" xlink:type="simple"/>
    </Attribution>
    <MetadataURL type="">
     <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="https://www.naturalearthdata.com/about/" xlink:type="simple"/>
    </MetadataURL>
    <TreeName>country-name</TreeName>
    <PrimaryKey>
     <PrimaryKeyAttribute>ogc_fid</PrimaryKeyAttribute>
    </PrimaryKey>
    <Attributes>
     <Attribute length="0" editType="TextEdit" type="qlonglong" comment="" precision="0" typeName="Integer64" name="ogc_fid"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="scalerank"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="labelrank"/>
     <Attribute length="0" editType="TextEdit" type="qlonglong" comment="" precision="0" typeName="Integer64" name="z_postal"/>
     <Attribute length="0" editType="TextEdit" type="qlonglong" comment="" precision="0" typeName="Integer64" name="z_abbrev"/>
     <Attribute length="0" editType="TextEdit" type="qlonglong" comment="" precision="0" typeName="Integer64" name="z_name"/>
     <Attribute length="0" editType="TextEdit" type="qlonglong" comment="" precision="0" typeName="Integer64" name="z_admin"/>
     <Attribute length="30" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="featurecla"/>
     <Attribute length="32" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="sovereignt"/>
     <Attribute length="3" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="sov_a3"/>
     <Attribute length="0" editType="TextEdit" type="double" comment="" precision="0" typeName="Real" name="adm0_dif"/>
     <Attribute length="0" editType="TextEdit" type="double" comment="" precision="0" typeName="Real" name="level"/>
     <Attribute length="17" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="type"/>
     <Attribute length="36" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="admin"/>
     <Attribute length="3" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="adm0_a3"/>
     <Attribute length="0" editType="TextEdit" type="double" comment="" precision="0" typeName="Real" name="geou_dif"/>
     <Attribute length="26" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name"/>
     <Attribute length="13" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="abbrev"/>
     <Attribute length="4" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="postal"/>
     <Attribute length="53" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_forma"/>
     <Attribute length="32" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="terr_"/>
     <Attribute length="43" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_sort"/>
     <Attribute length="0" editType="TextEdit" type="double" comment="" precision="0" typeName="Real" name="map_color"/>
     <Attribute length="0" editType="TextEdit" type="double" comment="" precision="0" typeName="Real" name="pop_est"/>
     <Attribute length="0" editType="TextEdit" type="double" comment="" precision="0" typeName="Real" name="gdp_md_est"/>
     <Attribute length="0" editType="TextEdit" type="double" comment="" precision="0" typeName="Real" name="fips_10_"/>
     <Attribute length="3" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="iso_a2"/>
     <Attribute length="3" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="iso_a3"/>
     <Attribute length="0" editType="TextEdit" type="double" comment="" precision="0" typeName="Real" name="iso_n3"/>
    </Attributes>
   </Layer>
   <Layer geometryType="MultiLineString" queryable="1" opacity="1" visible="1" expanded="1" displayField="name">
    <Name>state</Name>
    <Title>state</Title>
    <CRS>CRS:84</CRS>
    <CRS>EPSG:3857</CRS>
    <CRS>EPSG:4326</CRS>
    <EX_GeographicBoundingBox>
     <westBoundLongitude>-139.060207</westBoundLongitude>
     <eastBoundLongitude>153.506568</eastBoundLongitude>
     <southBoundLatitude>-39.201702</southBoundLatitude>
     <northBoundLatitude>78.686917</northBoundLatitude>
    </EX_GeographicBoundingBox>
    <BoundingBox maxy="153.506568" miny="-139.060207" CRS="EPSG:4326" maxx="78.686917" minx="-39.201702"/>
    <BoundingBox maxy="14747260.316" miny="-4750604.908" CRS="EPSG:3857" maxx="17088272.979" minx="-15480111.391"/>
    <Style>
     <Name>default</Name>
     <Title>default</Title>
     <LegendURL>
      <Format>image/png</Format>
      <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?&amp;SERVICE=WMS&amp;VERSION=1.3.0&amp;REQUEST=GetLegendGraphic&amp;LAYER=state&amp;FORMAT=image/png&amp;STYLE=default&amp;SLD_VERSION=1.1.0" xlink:type="simple"/>
     </LegendURL>
    </Style>
    <TreeName>state</TreeName>
    <PrimaryKey>
     <PrimaryKeyAttribute>ogc_fid</PrimaryKeyAttribute>
    </PrimaryKey>
    <Attributes>
     <Attribute length="0" editType="TextEdit" type="qlonglong" comment="" precision="0" typeName="Integer64" name="ogc_fid"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="featurecla"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="adm0_a3"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="adm0_name"/>
     <Attribute length="0" editType="TextEdit" type="double" comment="" precision="0" typeName="Real" name="shape_leng"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="mapcolor13"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="mapcolor9"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="sov_a3"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_l"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_r"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_alt_l"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_alt_r"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_loc_l"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_loc_r"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="name_len_l"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="name_len_r"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="note"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="type"/>
     <Attribute length="0" editType="TextEdit" type="double" comment="" precision="0" typeName="Real" name="min_zoom"/>
     <Attribute length="0" editType="TextEdit" type="double" comment="" precision="0" typeName="Real" name="min_label"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="scalerank"/>
    </Attributes>
   </Layer>
   <Layer geometryType="MultiPolygon" queryable="1" opacity="1" visible="1" expanded="0" displayField="name">
    <Name>country</Name>
    <Title>country</Title>
    <CRS>CRS:84</CRS>
    <CRS>EPSG:3857</CRS>
    <CRS>EPSG:4326</CRS>
    <EX_GeographicBoundingBox>
     <westBoundLongitude>-179.999926</westBoundLongitude>
     <eastBoundLongitude>179.999926</eastBoundLongitude>
     <southBoundLatitude>-89.501388</southBoundLatitude>
     <northBoundLatitude>83.634081</northBoundLatitude>
    </EX_GeographicBoundingBox>
    <BoundingBox maxy="179.999926" miny="-179.999926" CRS="EPSG:4326" maxx="83.634081" minx="-89.501388"/>
    <BoundingBox maxy="18428900" miny="-34679800" CRS="EPSG:3857" maxx="20037500" minx="-20037500"/>
    <Style>
     <Name>default</Name>
     <Title>default</Title>
     <LegendURL>
      <Format>image/png</Format>
      <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?&amp;SERVICE=WMS&amp;VERSION=1.3.0&amp;REQUEST=GetLegendGraphic&amp;LAYER=country&amp;FORMAT=image/png&amp;STYLE=default&amp;SLD_VERSION=1.1.0" xlink:type="simple"/>
     </LegendURL>
    </Style>
    <TreeName>country</TreeName>
    <PrimaryKey>
     <PrimaryKeyAttribute>ogc_fid</PrimaryKeyAttribute>
    </PrimaryKey>
    <Attributes>
     <Attribute length="0" editType="TextEdit" type="qlonglong" comment="" precision="0" typeName="Integer64" name="ogc_fid"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="featurecla"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="scalerank"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="labelrank"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="sovereignt"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="sov_a3"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="adm0_dif"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="level"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="type"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="admin"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="adm0_a3"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="geou_dif"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="geounit"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="gu_a3"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="su_dif"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="subunit"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="su_a3"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="brk_diff"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_long"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="brk_a3"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="brk_name"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="brk_group"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="abbrev"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="postal"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="formal_en"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="formal_fr"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_ciawf"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="note_adm0"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="note_brk"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_sort"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_alt"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="mapcolor7"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="mapcolor8"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="mapcolor9"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="mapcolor13"/>
     <Attribute length="0" editType="TextEdit" type="qlonglong" comment="" precision="0" typeName="Integer64" name="pop_est"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="pop_rank"/>
     <Attribute length="0" editType="TextEdit" type="double" comment="" precision="0" typeName="Real" name="gdp_md_est"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="pop_year"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="lastcensus"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="gdp_year"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="economy"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="income_grp"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="wikipedia"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="fips_10_"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="iso_a2"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="iso_a3"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="iso_a3_eh"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="iso_n3"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="un_a3"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="wb_a2"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="wb_a3"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="woe_id"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="woe_id_eh"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="woe_note"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="adm0_a3_is"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="adm0_a3_us"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="adm0_a3_un"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="adm0_a3_wb"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="continent"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="region_un"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="subregion"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="region_wb"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="name_len"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="long_len"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="abbrev_len"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="tiny"/>
     <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="homepart"/>
     <Attribute length="0" editType="TextEdit" type="double" comment="" precision="0" typeName="Real" name="min_zoom"/>
     <Attribute length="0" editType="TextEdit" type="double" comment="" precision="0" typeName="Real" name="min_label"/>
     <Attribute length="0" editType="TextEdit" type="double" comment="" precision="0" typeName="Real" name="max_label"/>
     <Attribute length="0" editType="TextEdit" type="qlonglong" comment="" precision="0" typeName="Integer64" name="ne_id"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="wikidataid"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_ar"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_bn"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_de"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_en"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_es"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_fr"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_el"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_hi"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_hu"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_id"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_it"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_ja"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_ko"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_nl"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_pl"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_pt"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_ru"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_sv"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_tr"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_vi"/>
     <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_zh"/>
    </Attributes>
   </Layer>
   <Layer queryable="1" visible="0" expanded="1" mutuallyExclusive="0">
    <Name>geo-lines</Name>
    <Title>geo-lines</Title>
    <CRS>CRS:84</CRS>
    <CRS>EPSG:3857</CRS>
    <CRS>EPSG:4326</CRS>
    <EX_GeographicBoundingBox>
     <westBoundLongitude>-179.999926</westBoundLongitude>
     <eastBoundLongitude>179.999926</eastBoundLongitude>
     <southBoundLatitude>-89.999996</southBoundLatitude>
     <northBoundLatitude>89.999996</northBoundLatitude>
    </EX_GeographicBoundingBox>
    <BoundingBox maxy="179.999926" miny="-179.999926" CRS="EPSG:4326" maxx="89.999996" minx="-89.999996"/>
    <BoundingBox maxy="109516377.539" miny="-109516377.571" CRS="EPSG:3857" maxx="20037500.106" minx="-20037500.106"/>
    <TreeName>geo-lines</TreeName>
    <Layer geometryType="MultiLineString" queryable="1" opacity="1" visible="0" expanded="1" displayField="name">
     <Name>ne_10m_geographic_lines</Name>
     <Title>ne_10m_geographic_lines</Title>
     <CRS>CRS:84</CRS>
     <CRS>EPSG:3857</CRS>
     <CRS>EPSG:4326</CRS>
     <EX_GeographicBoundingBox>
      <westBoundLongitude>-179.999926</westBoundLongitude>
      <eastBoundLongitude>179.999926</eastBoundLongitude>
      <southBoundLatitude>-89.999996</southBoundLatitude>
      <northBoundLatitude>89.999996</northBoundLatitude>
     </EX_GeographicBoundingBox>
     <BoundingBox maxy="179.999926" miny="-179.999926" CRS="EPSG:4326" maxx="89.999996" minx="-89.999996"/>
     <BoundingBox maxy="108777000" miny="-108777000" CRS="EPSG:3857" maxx="20037500" minx="-20037500"/>
     <Style>
      <Name>default</Name>
      <Title>default</Title>
      <LegendURL>
       <Format>image/png</Format>
       <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?&amp;SERVICE=WMS&amp;VERSION=1.3.0&amp;REQUEST=GetLegendGraphic&amp;LAYER=ne_10m_geographic_lines&amp;FORMAT=image/png&amp;STYLE=default&amp;SLD_VERSION=1.1.0" xlink:type="simple"/>
      </LegendURL>
     </Style>
     <MinScaleDenominator>0</MinScaleDenominator>
     <MaxScaleDenominator>5e+06</MaxScaleDenominator>
     <TreeName>ne_10m_geographic_lines</TreeName>
     <PrimaryKey>
      <PrimaryKeyAttribute>ogc_fid</PrimaryKeyAttribute>
     </PrimaryKey>
     <Attributes>
      <Attribute length="0" editType="" type="qlonglong" comment="" precision="0" typeName="Integer64" name="ogc_fid"/>
      <Attribute length="0" editType="" type="qlonglong" comment="" precision="0" typeName="Integer64" name="scalerank"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_long"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="abbrev"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="note"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="featurecla"/>
      <Attribute length="0" editType="" type="double" comment="" precision="0" typeName="Real" name="min_zoom"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="wikidataid"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_ar"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_bn"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_de"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_en"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_es"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_fr"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_el"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_hi"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_hu"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_id"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_it"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_ja"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_ko"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_nl"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_pl"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_pt"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_ru"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_sv"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_tr"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_vi"/>
      <Attribute length="0" editType="" type="QString" comment="" precision="0" typeName="String" name="name_zh"/>
      <Attribute length="0" editType="" type="int" comment="" precision="0" typeName="Integer" name="wdid_score"/>
      <Attribute length="0" editType="" type="qlonglong" comment="" precision="0" typeName="Integer64" name="ne_id"/>
     </Attributes>
    </Layer>
    <Layer geometryType="MultiLineString" queryable="1" opacity="1" visible="0" expanded="1" displayField="name">
     <Name>ne_50m_geographic_lines</Name>
     <Title>ne_50m_geographic_lines</Title>
     <CRS>CRS:84</CRS>
     <CRS>EPSG:3857</CRS>
     <CRS>EPSG:4326</CRS>
     <EX_GeographicBoundingBox>
      <westBoundLongitude>-179.999926</westBoundLongitude>
      <eastBoundLongitude>179.999926</eastBoundLongitude>
      <southBoundLatitude>-89.999996</southBoundLatitude>
      <northBoundLatitude>89.999996</northBoundLatitude>
     </EX_GeographicBoundingBox>
     <BoundingBox maxy="179.999926" miny="-179.999926" CRS="EPSG:4326" maxx="89.999996" minx="-89.999996"/>
     <BoundingBox maxy="108777000" miny="-108777000" CRS="EPSG:3857" maxx="20037500" minx="-20037500"/>
     <Style>
      <Name>default</Name>
      <Title>default</Title>
      <LegendURL>
       <Format>image/png</Format>
       <OnlineResource xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="http://127.0.0.1:8080/wms/qgs/ne?&amp;SERVICE=WMS&amp;VERSION=1.3.0&amp;REQUEST=GetLegendGraphic&amp;LAYER=ne_50m_geographic_lines&amp;FORMAT=image/png&amp;STYLE=default&amp;SLD_VERSION=1.1.0" xlink:type="simple"/>
      </LegendURL>
     </Style>
     <MinScaleDenominator>5e+06</MinScaleDenominator>
     <MaxScaleDenominator>1e+08</MaxScaleDenominator>
     <TreeName>ne_50m_geographic_lines</TreeName>
     <PrimaryKey>
      <PrimaryKeyAttribute>ogc_fid</PrimaryKeyAttribute>
     </PrimaryKey>
     <Attributes>
      <Attribute length="0" editType="TextEdit" type="qlonglong" comment="" precision="0" typeName="Integer64" name="ogc_fid"/>
      <Attribute length="0" editType="TextEdit" type="qlonglong" comment="" precision="0" typeName="Integer64" name="scalerank"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_long"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="abbrev"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="note"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="featurecla"/>
      <Attribute length="0" editType="TextEdit" type="double" comment="" precision="0" typeName="Real" name="min_zoom"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="wikidataid"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_ar"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_bn"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_de"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_en"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_es"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_fr"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_el"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_hi"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_hu"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_id"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_it"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_ja"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_ko"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_nl"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_pl"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_pt"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_ru"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_sv"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_tr"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_vi"/>
      <Attribute length="0" editType="TextEdit" type="QString" comment="" precision="0" typeName="String" name="name_zh"/>
      <Attribute length="0" editType="Range" type="int" comment="" precision="0" typeName="Integer" name="wdid_score"/>
      <Attribute length="0" editType="TextEdit" type="qlonglong" comment="" precision="0" typeName="Integer64" name="ne_id"/>
     </Attributes>
    </Layer>
   </Layer>
  </Layer>
  <LayerDrawingOrder>ne_50m_geographic_lines,ne_10m_geographic_lines,country,state,ne</LayerDrawingOrder>
 </Capability>
</WMS_Capabilities>
    "##;
        let cap: WmsCapabilities = from_reader(s.as_bytes()).unwrap();
        println!("{:#?}", cap);

        assert_eq!(cap.version, "1.3.0");
        assert_eq!(cap.service.name, "WMS");
        assert_eq!(cap.service.title, "untitled");

        assert_eq!(cap.capability.layers[0].name, None);
        assert_eq!(cap.capability.layers[0].title, None);
        assert_eq!(cap.capability.layers[0].queryable, Some(true));
        assert_eq!(cap.capability.layers[0].abstract_, None);
        assert_eq!(
            cap.capability.layers[0].crs,
            vec!["CRS:84", "EPSG:3857", "EPSG:4326"]
        );

        let layer = &cap.capability.layers[0].layers[0];
        assert_eq!(layer.name, Some("ne".to_string()));
        assert_eq!(layer.title, Some("Natural Earth".to_string()));
        assert_eq!(layer.crs, vec!["CRS:84", "EPSG:3857", "EPSG:4326"]);
        assert_eq!(layer.attribution.as_ref().unwrap().title, "Natural Earth");
        assert_eq!(
            layer.attribution.as_ref().unwrap().online_resource.href,
            "https://www.naturalearthdata.com/"
        );
        assert_eq!(
            layer.data_url.as_ref().unwrap().online_resource.href,
            "https://www.naturalearthdata.com/"
        );

        // <Layer opacity="1" visible="1" queryable="1" displayField="z_name" geometryType="Point" expanded="1">
        assert_eq!(layer.opacity, Some(1.0));
        assert_eq!(layer.visible, Some(true));
        assert_eq!(layer.display_field, Some("z_name".to_string()));
        assert_eq!(layer.geometry_type, Some("Point".to_string()));
        assert_eq!(layer.expanded, Some(true));

        let layer = &cap.capability.layers[0].layers[3];
        assert_eq!(layer.name, Some("geo-lines".to_string()));
        // <Layer visible="0" queryable="1" expanded="1" mutuallyExclusive="0">
        assert_eq!(layer.mutually_exclusive, Some(false));
    }
}
