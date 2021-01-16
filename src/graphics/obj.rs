use super::{Mesh, Vertex};
use cgmath::{Point2, Point3, Vector4};
use std::fs;
use std::str::FromStr;

lazy_static! {
    static ref PALLETE: Vec<(Vector4<f32>, Point3<f32>)> = vec![
        //Row 1
        color_pallette(0.0, 0.0, 196.0, 217.0, 225.0),
        color_pallette(1.0, 0.0, 151.0, 176.0, 186.0),
        color_pallette(2.0, 0.0, 101.0, 120.0, 127.0),
        color_pallette(3.0, 0.0, 55.0, 66.0, 71.0),
        color_pallette(4.0, 0.0, 255.0, 217.0, 193.0),
        color_pallette(5.0, 0.0, 247.0, 191.0, 177.0),
        color_pallette(6.0, 0.0, 214.0, 162.0, 163.0),
        color_pallette(7.0, 0.0, 140.0, 102.0, 125.0),

        //Row 2
        color_pallette(0.0, 1.0, 120.0, 180.0, 254.0),
        color_pallette(1.0, 1.0, 44.0, 138.0, 251.0),
        color_pallette(2.0, 1.0, 10.0, 97.0, 203.0),
        color_pallette(3.0, 1.0, 3.0, 50.0, 106.0),
        color_pallette(4.0, 1.0, 120.0, 254.0, 122.0),
        color_pallette(5.0, 1.0, 53.0, 231.0, 29.0),
        color_pallette(6.0, 1.0, 26.0, 151.0, 9.0),
        color_pallette(7.0, 1.0, 11.0, 76.0, 2.0),

        //Row 3
        color_pallette(0.0, 2.0, 246.0, 218.0, 248.0),
        color_pallette(1.0, 2.0, 239.0, 168.0, 246.0),
        color_pallette(2.0, 2.0, 229.0, 84.0, 243.0),
        color_pallette(3.0, 2.0, 173.0, 7.0, 189.0),
        color_pallette(4.0, 2.0, 211.0, 175.0, 247.0),
        color_pallette(5.0, 2.0, 164.0, 101.0, 226.0),
        color_pallette(6.0, 2.0, 125.0, 58.0, 191.0),
        color_pallette(7.0, 2.0, 72.0, 11.0, 131.0),

        //Row 4
        color_pallette(0.0, 3.0, 248.0, 213.0, 201.0),
        color_pallette(1.0, 3.0, 241.0, 188.0, 169.0),
        color_pallette(2.0, 3.0, 317.0, 161.0, 123.0),
        color_pallette(3.0, 3.0, 190.0, 145.0, 108.0),
        color_pallette(4.0, 3.0, 123.0, 91.0, 65.0),
        color_pallette(5.0, 3.0, 98.0, 62.0, 43.0),
        color_pallette(6.0, 3.0, 67.0, 40.0, 26.0),
        color_pallette(7.0, 3.0, 37.0, 22.0, 10.0),

        //Row 5
        color_pallette(0.0, 4.0, 255.0, 198.0, 76.0),
        color_pallette(1.0, 4.0, 253.0, 160.0, 0.0),
        color_pallette(2.0, 4.0, 240.0, 121.0, 0.0),
        color_pallette(3.0, 4.0, 198.0, 86.0, 0.0),
        color_pallette(4.0, 4.0, 217.0, 185.0, 157.0),
        color_pallette(4.0, 4.0, 189.0, 151.0, 117.0),
        color_pallette(6.0, 4.0, 146.0, 104.0, 66.0),
        color_pallette(7.0, 4.0, 101.0, 69.0, 29.0),

        //Row 6
        color_pallette(0.0, 5.0, 251.0, 180.0, 161.0),
        color_pallette(1.0, 5.0, 220.0, 90.0, 58.0),
        color_pallette(2.0, 5.0, 171.0, 58.0, 29.0),
        color_pallette(3.0, 5.0, 126.0, 26.0, 9.0),
        color_pallette(4.0, 5.0, 249.0, 243.0, 166.0),
        color_pallette(5.0, 5.0, 248.0, 227.0, 37.0),
        color_pallette(6.0, 5.0, 223.0, 183.0, 10.0),
        color_pallette(7.0, 5.0, 175.0, 144.0, 0.0),

        //Row 7
        color_pallette(0.0, 6.0, 193.0, 234.0, 225.0),
        color_pallette(1.0, 6.0, 142.0, 198.0, 226.0),
        color_pallette(2.0, 6.0, 75.0, 147.0, 184.0),
        color_pallette(3.0, 6.0, 24.0, 86.0, 118.0),
        color_pallette(4.0, 6.0, 211.0, 233.0, 166.0),
        color_pallette(5.0, 6.0, 156.0, 188.0, 98.0),
        color_pallette(6.0, 6.0, 102.0, 145.0, 50.0),
        color_pallette(7.0, 6.0, 54.0, 91.0, 19.0),

        //Row 8
        color_pallette(0.0, 7.0, 245.0, 245.0, 245.0),
        color_pallette(1.0, 7.0, 228.0, 228.0, 228.0),
        color_pallette(2.0, 7.0, 206.0, 206.0, 206.0),
        color_pallette(3.0, 7.0, 177.0, 177.0, 177.0),
        color_pallette(4.0, 7.0, 142.0, 142.0, 142.0),
        color_pallette(5.0, 7.0, 102.0, 102.0, 102.0),
        color_pallette(6.0, 7.0, 62.0, 62.0, 62.0),
        color_pallette(7.0, 7.0, 5.0, 5.0, 5.0)
    ];
}

/// Loads a mesh from the assets using the default pallete
pub fn load_mesh(name: &str) -> Mesh {
    let mut mesh = Mesh {
        name: name.to_string(),
        vertices: Vec::new(),
        indices: Vec::new(),
    };
    let path = format!("assets/models/{}.obj", name);

    let text = fs::read_to_string(path).expect("Unable to load mesh!");
    let obj = parse_obj_file(text);

    for face in &obj.faces {
        let v1 = add_vertex(&mut mesh, &obj, &face.x);
        let v2 = add_vertex(&mut mesh, &obj, &face.y);
        let v3 = add_vertex(&mut mesh, &obj, &face.z);

        mesh.indices.push(v1 as u16);
        mesh.indices.push(v2 as u16);
        mesh.indices.push(v3 as u16);
    }

    mesh
}

fn add_vertex(mesh: &mut Mesh, obj_data: &ObjData, vertex: &ObjVertex) -> usize {
    mesh.vertices.push(Vertex {
        pos: obj_data.vertices[vertex.v - 1],
        normal: obj_data.normals[vertex.vn - 1],
        color: get_color(obj_data.colors[vertex.vt - 1]),
    });

    mesh.vertices.len() - 1
}

fn get_color(pt: Point2<f32>) -> Point3<f32> {
    for element in PALLETE.iter() {
        let (rect, color) = element;

        if rect.x < pt.x && rect.y < pt.y && rect.z > pt.x && rect.w > pt.y {
            return *color;
        }
    }

    panic!("Invalid color: {:?}", pt);
}

#[allow(clippy::many_single_char_names)]
fn color_pallette(x: f32, y: f32, r: f32, g: f32, b: f32) -> (Vector4<f32>, Point3<f32>) {
    let x0 = x / 8.0;
    let y0 = y / 8.0;
    let x1 = (x + 1.0) / 8.0;
    let y1 = (y + 1.0) / 8.0;

    (
        Vector4::new(x0, y0, x1, y1),
        Point3::new(r / 255.0, g / 255.0, b / 255.0),
    )
}

struct ObjData {
    vertices: Vec<Point3<f32>>,
    colors: Vec<Point2<f32>>,
    normals: Vec<Point3<f32>>,
    faces: Vec<Point3<ObjVertex>>,
}

struct ObjVertex {
    v: usize,
    vt: usize,
    vn: usize,
}

fn parse_obj_file(text: String) -> ObjData {
    let mut data = ObjData {
        vertices: Vec::new(),
        colors: Vec::new(),
        normals: Vec::new(),
        faces: Vec::new(),
    };

    for line in text.lines() {
        let mut words = line.split_whitespace();

        match words.next() {
            Some("#") | Some("o") | Some("s") | None => continue,
            Some("v") => {
                let x = parse_float(words.next());
                let y = parse_float(words.next());
                let z = parse_float(words.next());

                data.vertices.push(Point3::new(x, y, z));
            }
            Some("vt") => {
                let u = parse_float(words.next());
                let v = parse_float(words.next());

                data.colors.push(Point2::new(u, v));
            }
            Some("vn") => {
                let x = parse_float(words.next());
                let y = parse_float(words.next());
                let z = parse_float(words.next());

                data.normals.push(Point3::new(x, y, z));
            }
            Some("f") => {
                let v1 = parse_obj_vertex(words.next());
                let v2 = parse_obj_vertex(words.next());
                let v3 = parse_obj_vertex(words.next());

                if words.next().is_some() {
                    panic!("Mesh not triangularized!");
                }

                data.faces.push(Point3::new(v1, v2, v3));
            }
            Some(x) => println!("Unknown line in obj: {}", x),
        }
    }

    data
}

fn parse_float(input: Option<&str>) -> f32 {
    FromStr::from_str(input.unwrap()).unwrap()
}

fn parse_obj_vertex(input: Option<&str>) -> ObjVertex {
    let mut parts = input.unwrap().split('/');
    let v = FromStr::from_str(parts.next().unwrap()).unwrap();
    let vt = FromStr::from_str(parts.next().unwrap()).unwrap();
    let vn = FromStr::from_str(parts.next().unwrap()).unwrap();

    ObjVertex { v, vt, vn }
}
