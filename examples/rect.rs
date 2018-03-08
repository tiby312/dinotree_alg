extern crate dinotree;
use dinotree::*;
use dinotree::support::Numisize;
use dinotree::support::BBox;
use dinotree::support::DefaultDepthLevel;

#[derive(Debug)]
struct Bot{
    id:usize
}

fn aabb(x:(isize,isize),y:(isize,isize))->AABBox<Numisize>{
  AABBox::new(  (Numisize(x.0),Numisize(x.1)),(Numisize(y.0),Numisize(y.1)))
}
fn make_bot(id:usize,x:(isize,isize),y:(isize,isize))->BBox<Numisize,Bot>{
	let rect=aabb(x,y);
	BBox::new(Bot{id},rect)
}

fn main(){

  	let mut bots:Vec<BBox<Numisize,Bot>>=Vec::new();

  	bots.push(make_bot(0,(10,20),(50,60)));
  	bots.push(make_bot(1,(15,25),(50,60)));
  	bots.push(make_bot(2,(16,30),(50,60)));



  	bots.push(make_bot(3,(50,60),(10,20)));
  	bots.push(make_bot(4,(50,60),(15,25)));
  	bots.push(make_bot(5,(50,60),(16,30)));


    {
        let mut dyntree=DinoTree::new(&mut bots,true);
       
        let mut rects=dyntree.rects();
        let r1=aabb((10,25),(40,70));
        let r2=aabb((40,70),(10,25));
          
        let mut bb1:Vec<&mut Bot>=Vec::new();
        let mut bb2:Vec<&mut Bot>=Vec::new();
        
        {
          rects.for_all_in_rect(&r1,
            |cc:ColSingle<BBox<Numisize,Bot>>|{
              bb1.push(cc.1);
          });
        }

        {
          rects.for_all_in_rect(&r2,
            |cc:ColSingle<BBox<Numisize,Bot>>|{
              bb2.push(cc.1);
          });
        }


        println!("These bots are in:{:?}",r1);
        for b in bb1.iter(){
          println!("\tbots={:?}",(&b.id));
        }

        println!("These bots are in:{:?}",r2);
        for b in bb2.iter(){
          println!("\tbots={:?}",(&b.id));
        }
	}
}
