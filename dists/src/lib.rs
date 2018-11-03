





/*
pub trait ObjectDistributionGenerator{
    type T:Iterator<Item=[f64;2]>+Clone;
    type I:Iterator<Item=Self::T>+Clone;
    fn create(&self)->Self::I;
}

pub trait ObjectDistribution{
    type I:Iterator<Item=[f64;2]>;
    fn create(&self)->Self::I;
}
*/

/*

pub mod clump{
    use super::*;

    pub struct ClumpDist{
        point:[f64;2],
        offset_increase_rate:f64
    }
    impl ClumpDist{
        pub fn new(point:[f64;2],offset_increase_rate:f64)->ClumpDist{
            ClumpDist{point,offset_increase_rate}
        }
    }
    impl ObjectDistributionGenerator for ClumpDist{
        type T=Clump;
        type I=ClumpGenerator;
        fn create(&self)->ClumpGenerator{
            ClumpGenerator{point:self.point,offset_increase_rate:self.offset_increase_rate,offset_rate:0.0}
        }
    }

    #[derive(Copy,Clone)]
    pub struct ClumpGenerator{
        offset_increase_rate:f64,
        offset_rate:f64,
        point:[f64;2]
    }
    impl std::iter::FusedIterator for ClumpGenerator{}

    impl Iterator for ClumpGenerator{
        type Item=Clump;
        fn next(&mut self)->Option<Self::Item>{
            let s=Clump{point:self.point,offset:0.0,offset_rate:self.offset_rate};
            self.offset_rate+=self.offset_increase_rate;
            Some(s)
        }
    }



    #[derive(Copy,Clone)]
    pub struct Clump{
        point:[f64;2],
        offset:f64,
        offset_rate:f64
    }
    impl Clump{
        pub fn new(point:[f64;2],offset_rate:f64)->Clump{
            Clump{point,offset:0.0,offset_rate}
        }
    }

    impl std::iter::FusedIterator for Clump{}
    impl ObjectDistribution for Clump{
        type I=Self;
        fn create(&self)->Clump{
            self.clone()
        }
    }
    impl Iterator for Clump{
        type Item=[f64;2];
        fn next(&mut self)->Option<Self::Item>{
            let k=Some([self.point[0]+self.offset,  self.point[1]+self.offset]);
            self.offset+=self.offset_rate;
            k
        }
    }
}
*/

pub mod spiral{
    use super::*;


    #[derive(Copy,Clone)]
    pub struct Spiral{
        point:[f64;2],
        rad:f64,
        start:f64,
        rate:f64,
        width:f64
    }

    impl Spiral{
        pub fn new(point:[f64;2],circular_grow:f64,outward_grow:f64)->Spiral{
            Spiral{point,rad:0.0,start:1.0,rate:outward_grow,width:circular_grow}
        }
        pub fn get_circular_grow(&self)->f64{
            self.width
        }
        pub fn get_outward_grow(&self)->f64{
            self.rate
        }
    }
    
    impl std::iter::FusedIterator for Spiral{}

    impl Iterator for Spiral{
        type Item=[f64;2];
        fn next(&mut self)->Option<[f64;2]>{
            
            let length=self.start+self.rate*self.rad;

            let x=self.point[0]+self.rad.cos()*length;
            let y=self.point[1]+self.rad.sin()*length;

            self.rad+=self.width/length;

            Some([x,y])

        }
    }

}

