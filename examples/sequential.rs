extern crate dinotree;
use dinotree::prelude::*;
use dinotree::support::Numisize;
use dinotree::support::BBox;
use dinotree::support::DefaultDepthLevel;


#[derive(Debug)]
struct Bot{
    id:usize,
    touching:Vec<usize> 
}


fn make_bot(id:usize,x:(isize,isize),y:(isize,isize))->BBox<Numisize,Bot>{
	let rect=AABBox::new(  (Numisize(x.0),Numisize(x.1)),(Numisize(y.0),Numisize(y.1)));
	BBox::new(Bot{id,touching:Vec::new()},rect)
}

fn main(){

  	let mut bots:Vec<BBox<Numisize,Bot>>=Vec::new();

  	bots.push(make_bot(0,(10,20),(50,60)));
  	bots.push(make_bot(1,(15,25),(50,60)));
  	bots.push(make_bot(2,(16,30),(50,60)));



  	bots.push(make_bot(3,(50,60),(10,20)));
  	bots.push(make_bot(4,(50,60),(15,25)));
  	bots.push(make_bot(5,(50,60),(16,30)));


  	let height=2;

  	let mut treecache:TreeCache2<Numisize>=TreeCache2::new(daxis::XAXIS,height);

    {
        let k=MedianStrict::<Numisize>::new();
        let (mut dyntree,_bag)=treecache.new_tree::<_,par::Sequential,DefaultDepthLevel,_,treetimer::TreeTimerEmpty>
                        (&mut bots,&k);
        

        let clos=|cc:ColPair<BBox<Numisize,Bot>>|{
        	cc.a.1.touching.push(cc.b.1.id);
        	cc.b.1.touching.push(cc.a.1.id);
        };

        let _v=dyntree.for_every_col_pair_seq::<_,treetimer::TreeTimer2>(clos);
         

	}

  println!("These bots are colliding:");
	for b in bots.iter(){
		println!("bots={:?}",(&b.val.id,&b.val.touching));
	}

}
