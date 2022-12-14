use crossbeam_utils::thread;
use std::arch::asm;

const BASE: u64 = ((b'C' as u64) << 24) + ((b'T' as u64) << 16);

#[inline(always)]
pub unsafe fn do_client_request(default: u64, args: &[u64; 6]) -> u64 {
    let result;
    asm!(
    "rol rdi, 3
        rol rdi, 13
        rol rdi, 61
        rol rdi, 51
        xchg rbx,rbx",
    inout("rdx") default=>result,
    in("rax") args.as_ptr()
    );
    result
}

#[inline(always)]
pub fn start() {
    let (type_value, first_argument) = (BASE + 4, 0);
    unsafe { do_client_request(0, &[type_value, first_argument, 0, 0, 0, 0]) };
    let (type_value, first_argument) = (BASE + 2, 0);
    unsafe { do_client_request(0, &[type_value, first_argument, 0, 0, 0, 0]) };
}

#[inline(always)]
pub fn stop() {
    let (type_value, first_argument) = (BASE + 2, 0);
    unsafe { do_client_request(0, &[type_value, first_argument, 0, 0, 0, 0]) };
    let (type_value, first_argument) = (BASE + 5, 0);
    unsafe { do_client_request(0, &[type_value, first_argument, 0, 0, 0, 0]) };
}

struct SubProduct1 {
    name: String,
    price: f64,
}
impl SubProduct1 {
    fn update_subproduct(&mut self) {
        process_name(&mut self.name);
        process_price(&mut self.price);
    }
}
struct SubProduct2 {
    vendor: String,
    production_site: String,
}
impl SubProduct2 {
    fn update_subproduct(&mut self) {
        process_vendor(&mut self.vendor);
        process_production_site(&mut self.production_site);
    }
}

struct Product {
    name: String,
    price: f64,
    vendor: String,
    production_site: String,
}

impl Product {
    fn into_subs(self) -> (SubProduct1, SubProduct2) {
        (
            SubProduct1 {
                name: self.name,
                price: self.price,
            },
            SubProduct2 {
                vendor: self.vendor,
                production_site: self.production_site,
            },
        )
    }

    fn update_product(&mut self) {
        process_name(&mut self.name);
        process_price(&mut self.price);
        process_vendor(&mut self.vendor);
        process_production_site(&mut self.production_site);
    }
}

fn process_name(name: &mut String) {
    if name == "Hamburger" {
        name.clear();
    }
}

fn process_price(price: &mut f64) {
    if *price > 50.0 {
        *price *= 0.8;
    }
}

fn process_vendor(vendor: &mut String) {
    if vendor != "Heinz" {
        vendor.push_str("Nevermind");
    }
}

fn process_production_site(production_site: &mut String) {
    if production_site == "Georgia" {
        *production_site = String::from("Montpellier")
    } else if production_site == "Afghanistan" {
        *production_site = String::from("Henry Cavill")
    } else {
        *production_site = String::from("World")
    }
}

fn update_1(products: &mut [Product]) {
    start();
    for product in products {
        product.update_product();
    }
    stop();
}

fn update_2(subpart1: &mut [Product], subpart2: &mut [Product]) {
    start();
    thread::scope(|scope| {
        let handler1 = scope.spawn(move |_| {
            start();
            for product in subpart1 {
                product.update_product();
            }
            stop();
        });
        let handler2 = scope.spawn(move |_| {
            start();
            for product in subpart2 {
                product.update_product();
            }
            stop();
        });
        handler1.join().unwrap();
        handler2.join().unwrap();
    })
    .unwrap();
    stop();
}

fn update_3(sproduct1s: &mut [SubProduct1], sproduct2s: &mut [SubProduct2]) {
    start();
    thread::scope(|scope| {
        let handler1 = scope.spawn(move |_| {
            start();
            for sproduct1 in sproduct1s {
                sproduct1.update_subproduct();
            }
            stop();
        });
        let handler2 = scope.spawn(move |_| {
            start();
            for sproduct2 in sproduct2s {
                sproduct2.update_subproduct();
            }
            stop();
        });
        handler1.join().unwrap();
        handler2.join().unwrap();
    })
    .unwrap();
    stop();
}

fn main() {
    let names: Vec<&str> = vec!["Ketchup", "Mustard", "Rice", "Hamburger", "Cream"];
    let prices: Vec<f64> = vec![1.45, 2.99, 6.9, 209.0, 55.2, 1.00, 0.0085];
    let vendors: Vec<&str> = vec!["Uncle Ben", "Heinz", "McDonald"];
    let production_sites: Vec<&str> = vec!["China", "Italy", "Afghanistan", "Georgia"];

    let mut products: Vec<Product> = (0..10_000_000)
        .into_iter()
        .map(|i| Product {
            name: String::from(names[i % names.len()]),
            price: prices[i % prices.len()],
            vendor: String::from(vendors[i % vendors.len()]),
            production_site: String::from(production_sites[i % production_sites.len()]),
        })
        .collect();

    let start = std::time::Instant::now();
    update_1(&mut products);
    println!("{:?}",start.elapsed());

    let (subpart1, subpart2) = products.split_at_mut(500_000);
    let start = std::time::Instant::now();
    update_2(subpart1, subpart2);
    println!("{:?}",start.elapsed());

    let (mut c1s, mut c2s): (Vec<SubProduct1>, Vec<SubProduct2>) = products
        .into_iter()
        .map(|product| product.into_subs())
        .unzip();
    let start = std::time::Instant::now();
    update_3(&mut c1s, &mut c2s);
    println!("{:?}",start.elapsed());
}
