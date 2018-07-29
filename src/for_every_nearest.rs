
use inner_prelude::*;
use k_nearest::Knearest;


///Here we exploit the fact that if the nearest point to a point A is B, then the nearest point to B is A.
///Finding the nearest distance between two shapes is difficult, and not implemented here.
///Finding the nearest distance betweeen two points is easy.
///By exploiting this propety that B.nearest()==A.nearest(), we can half the numbers of knearest() queries that need to be done.
///This function is implemented simply, by iterating thorugh all the bots and calling knearest on it.
///I think there room for improvement here. I think it can be turned into a divider and conquer type problem.
///But it is difficult to know which nodes to exclude. The "nearest" is specifcally a 2d problem. hard to split into 1d.
pub fn for_every_nearest_mut<A:AxisTrait,N:NumTrait,T:IsPoint<Num=N>+HasAabb<Num=N>,K:Knearest<T=T,N=N>+Copy>(tree:&mut DynTree<A,(),T>,mut func:impl FnMut(&mut T,&mut T,K::D),kn:K){
	let mut already_hit:Vec<*const T>=Vec::with_capacity(tree.get_num_bots()/2);

	let tree2=tree as *mut DynTree<A,(),T>;
	for b in tree.iter_every_bot_mut(){
        if !already_hit.contains(&(b as *const T)){	        
	        
	        let mut nearest_bot=None;

	        //Safe to do since we are mutating the nearest bot that is not the current bot.
	        //Safe if HasCenter and Knear
	        let tree2=unsafe{&mut *tree2};
	        

	        //We query for the 2 nearest because one that will be returned is itself.
			//If the current bot and its nearest are on top of each other,
	        //its entirely possible for the current bot to be returned second.
	        for (a,bb) in k_nearest::k_nearest_mut(tree2,b.get_center(),2,kn){
        		if a as *const T!=b as *const T{
        			nearest_bot=Some((a,bb))
        		}
	        }

	        if let Some(nb)=nearest_bot{
	        	func(b,nb.0,nb.1);
	        	already_hit.push(nb.0 as *const T);
	        }   
		}
    }

    //assert_eq!(already_hit.len(),tree.tree.get_num_bots()/2);
}


/*
fn handle_right(bot,anchor){

	fn go_down_right(){

	}
}
fn go_down_left(){
	//
	for bot in node{
		//considers bots in root and right of root.
	}

	match nn{
		Some((left,right))=>{
			let (further,nearest)=conv(left,right);
			//Use 2d
			let min_dis=rect.as_axis().get(axis.next());
			if min_dis<anchor.div<this.div{
				break;
			}else{
				go_down(further,axis.next());
			}
			go_down(nearer,axis.next());		
		}
	}
}

fn recc(){

	//find knearest for bots in this node.
	//if the nearest is in this divider, mark it so we dont hit it again.
	//if its not in the divider, we dont care. we wont hit it again.


	handle_all_nodes();

	let anchor=anchor;

	match nn{
		Some((left,right))=>{
			{
				let left=left.create_wrap();
				let right=right.create_wrap();
				go_down(div,left,|left,right|{left,right});
				go_down(div,right,|left,right|{right,left});
			}
			recc(left);
			recc(right);
		},
		None=>{

		}
	}
}
*/
/*

    let lines=tree.iter().map(|b|{
        let mut vv:Vec<(&T,knearf64::DisSqr)>=Vec::new();
        let mut counter=0;
        let nearest_bot=None;
        k_nearest::k_nearest(&tree,b.get_center(),2,kn,|a,b|{
        	if counter==1{
        		nearest_bot=Some((a,b))
        	}
        	counter+=1;
        });
        

        let b1=&b.inner;
        let b2=&vv[1].0.inner;
        (b1,b2)
    }).fold(||Vec::new(),|mut vec1,item|{vec1.push(item);vec1}).reduce(||Vec::new(),|mut a,mut b|{a.append(&mut b);a});

    for (b,b2) in lines.iter(){
        let arr=[b.pos[0] ,b.pos[1],b2.pos[0],b2.pos[1]];
        line([1.0, 0.0, 1.0, 0.2], // black
             1.0, // radius of line
             arr, // [x0, y0, x1,y1] coordinates of line
             c.transform,
             g);
    }
    */

/*
The design goal here is:

for ever node:
	for each bot in node{
		match knearest(bot){
			//if nearest is not in this node, then handle that pair.
			//if nearest is in this node, and we havent already handled int ,handle it (so we need memory)
		}
	}
	//at this point we have handled all the bots in ths node.

	//now we have to handle the cases where a bot on the left of the divider of this node has a nearest bot that is on the left side.


	recurse left. 
		If we encounter a divider that is aligned the same as the root do the following:
			if for all the bots in this node, I can find a bot that is closer than the distance between this nodes divider at the anchor,
			then we can stop recursing away from the annchor.

*/