extern crate crossbeam;
extern crate num;
use num::Complex;
/// Пытается определить, принадлежит ли `c` множеству Мандельброта, ограничившись
/// `limit` итерациями.
///
/// Если `c` не принадлежит множеству, вернуть `Some(i)`, где `i` – число итераций,
/// понадобившееся для того, чтобы `c` покинула круг радиуса 2 с центром в начале
/// координат. Если `c` может принадлежать множеству (точнее, если после limit итераций
/// не удалось доказать, что `c` не является элементом множества), то вернуть `None`.
fn escape_time(c: Complex<f64>, limit: u32) -> Option<u32> {
		let mut z = Complex { re: 0.0, im: 0.0 };
		for i in 0..limit {
				z = z*z + c;
				if z.norm_sqr() > 4.0 {
						return Some(i);
				}
		}
		None
}
use std::str::FromStr;
/// Разбирает строку `s`, содержащую пару координат, например: `"400x600"` или
/// `"1.0,0.5"`.
///
/// Точнее, `s` должна иметь вид <left><sep><right>, где <sep> – символ, заданный
/// в аргументе `separator`, а <left> и <right> – строки, допускающие разбор
/// методом `T::from_str`.
///
/// Если `s` удалось разобрать, то возвращает `Some<(x, y)>`, в противном случае
/// `None`.
fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
		match s.find(separator) {
				None => None,
				Some(index) => {
						match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
								(Ok(l), Ok(r)) => Some((l, r)),
								_ => None
						}
				}
            }
        }
        #[test]
        fn test_parse_pair() {
                assert_eq!(parse_pair::<i32>("",
         ','), None);
                assert_eq!(parse_pair::<i32>("10,",
         ','), None);
                assert_eq!(parse_pair::<i32>(",10",
         ','), None);
                assert_eq!(parse_pair::<i32>("10,20", ','), Some((10, 20)));
                assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
                assert_eq!(parse_pair::<f64>("0.5x",
         'x'), None);
                assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
        }
/// Разбирает пару чисел с плавающей точкой, разделенных запятой, и возвращает
/// ее в виде комплексного числа.
fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
            Some((re, im)) => Some(Complex { re, im }),
            None => None
    }
}
#[test]
fn test_parse_complex() {
    assert_eq!(parse_complex("1.25,-0.0625"),
    
Some(Complex { re: 1.25, im: -0.0625 }));
    assert_eq!(parse_complex(",-0.0625"), None);
}
/// Зная строку и столбец пикселя выходного изображения, возвращает соответствующую
/// точку на комплексной плоскости.
///
/// `bounds` - пара, определяющая ширину и высоту изображения в пикселях.
/// `pixel` - пара (строка, столбец), определяющая конкретный пиксель изображения.
/// Параметры `upper_left` и `lower_right` - точки на комплексной плоскости,
/// описывающие область, покрываемую изображением.
fn pixel_to_point(bounds: (usize, usize),
pixel: (usize, usize),
upper_left: Complex<f64>,
lower_right: Complex<f64>)
		-> Complex<f64>
{
		let (width, height) = (lower_right.re - upper_left.re,
		
 upper_left.im - lower_right.im);
		Complex {
				re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
				im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64
				 // Почему здесь вычитание? pixel.1 увеличивается при движении вниз,
				 // тогда как мнимая часть увеличивается при движении вверх.
		}
}
#[test]
fn test_pixel_to_point() {
		assert_eq!(pixel_to_point((100, 100), (25, 75),
		
 Complex { re: -1.0, im: 1.0 },
		
 Complex { re: 1.0, im: -1.0 }),
		
 Complex { re: -0.5, im: -0.5 });
}
/// Рисует прямоугольную часть множества Мандельброта в буфере пикселей.
///
/// Аргумент `bounds` задает ширину и высоту буфера `pixels`, в котором каждый байт
/// представляет один полутоновый пиксель. Аргументы `upper_left` и `lower_right`
/// определяют точки на комплексной плоскости, соответствующие левому верхнему
/// и правому нижнему углам буфера пикселей.
fn render(pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>)
    {
            assert!(pixels.len() == bounds.0 * bounds.1);
            for row in 0 .. bounds.1 {
                    for column in 0 .. bounds.0 {
                            let point = pixel_to_point(bounds, (column, row),
                            
     upper_left, lower_right);
                            pixels[row * bounds.0 + column] =
                                    match escape_time(point, 255) {
                                            None => 0,
                                            Some(count) => 255 - count as u8
                                    };
                    }
            }
    }
    extern crate image;
use image::ColorType;
use image::png::PNGEncoder;
use std::fs::File;
/// Записывает буфер `pixels`, размеры которого заданы аргументом `bounds`, в файл
/// с именем `filename`.
fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize))
		-> Result<(), std::io::Error>
{
		let output = File::create(filename)?;
		let encoder = PNGEncoder::new(output);
		encoder.encode(&pixels,
		
 bounds.0 as u32, bounds.1 as u32,
		
 ColorType::Gray(8))?;
		Ok(())
}
use std::io::Write;
fn main() {
		let args: Vec<String> = std::env::args().collect();
		if args.len() != 5 {
				writeln!(std::io::stderr(),
				
 "Порядок вызова: mandelbrot FILE PIXELS UPPERLEFT LOWERRIGHT")
						.unwrap();
				writeln!(std::io::stderr(),
				
 "Пример: {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
				
 args[0])
						.unwrap();
				std::process::exit(1);
		}
		let bounds = parse_pair(&args[2], 'x')
				.expect("ошибка при разборе размеров изображения");
		let upper_left = parse_complex(&args[3])
				.expect("ошибка при разборе координат левого верхнего угла");
		let lower_right = parse_complex(&args[4])
				.expect("ошибка при разборе координат правого нижнего угла");
		let mut pixels = vec![0; bounds.0 * bounds.1];
		render(&mut pixels, bounds, upper_left, lower_right);
		write_image(&args[1], &pixels, bounds)
				.expect("ошибка при записи PNG-файла");
}