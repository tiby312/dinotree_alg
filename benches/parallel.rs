#![feature(test)]

extern crate test;

use test::Bencher;

#[bench]
fn parallel(a:&mut Bencher){
	println!("al");
}

#[bench]
fn sequential(a:&mut Bencher){
	println!("al");
}
