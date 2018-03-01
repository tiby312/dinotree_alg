//!Provides capability to draw the dividers of each node. 
use inner_prelude::*;
use support::Numf32;
use compt::GenTree;



pub trait Vertex: std::default::Default+std::clone::Clone+Send{
    fn set_pos(&mut self,x:f32,y:f32);
}


///Adds functionality to draw the state of the tree.
pub struct GenTreeGraphics {

}
impl GenTreeGraphics {

    pub fn get_num_verticies(height:usize)->usize{
        let num_nodes=compt::compute_num_nodes(height);
        (num_nodes/2)*6
    }

    pub fn update<V:Vertex>(rect:axgeom::Rect<Numf32>,gentree: &TreeCache2<Numf32>,verticies:&mut [V],start_width:f32) {
        Self::update2(rect,gentree,verticies,start_width);
    }


    ///Updates the slice of verticies to reflect the state of the kdtree.
    ///Every median at every level is drawn as a line.
    ///Lines are drawn using 6 verticies as a trianglist.
    fn update2<V:Vertex>(rect:axgeom::Rect<Numf32>,gentree: &TreeCache2<Numf32>,verticies:&mut [V],start_width:f32) {

        struct Node<'a, V:Vertex+'a>{
            a:&'a mut [V]
        };

        let a=Self::get_num_verticies(gentree.get_tree().get_height());
        let b=verticies.len();
        assert_eq!( a,b);

        let height=gentree.get_tree().get_height();
        let mut vert_tree={
            let mut va=verticies;
            let nodes:GenTree<Node<V>>=GenTree::from_bfs(&mut ||{
                let v=std::mem::replace(&mut va,&mut []);
                let (a,b)=v.split_at_mut(6);

                std::mem::replace(&mut va,b);

                Node{a:a}
            
            },gentree.get_tree().get_height()-1);
            nodes
        };


        let level=gentree.get_tree().get_level_desc();
        let d1=gentree.get_tree().create_down();
        let d2=vert_tree.create_down_mut();
        let zip=compt::LevelIter::new(d1.zip(d2),level);
        //let ddd=DivAxisIter::new(gentree.get_starting_axis(),zip);
        
        fn recc<'a,V:Vertex+'a,D:CTreeIterator<Item=(&'a DivNode<Numf32>,&'a mut Node<'a,V>)>>
            (axis:axgeom::Axis,height:usize,rect:Rect<Numf32>,d:LevelIter<D>,width:f32)
            {
                //let div_axis=A::get();
                let div_axis=axis;
                match d.next(){
                    ((dd,nn),Some((left,right)))=>{
                        let line_axis=axis.next();

                        //let line_axis=A::Next::get();//div_axis.get_line();
                        let range=rect.get_range(line_axis);
                        draw_node(height,*range,nn.0,(div_axis,dd),nn.1.a,width);
                        
                        let (b, c) = rect.subdivide(*nn.0.divider(), div_axis);

                        recc::<_,_>(axis.next(),height,b,left,width*0.9);
                        recc::<_,_>(axis.next(),height,c,right,width*0.9);
                    },
                    ((_dd,_nn),None)=>{

                    }
                }

            }
        recc::<_,_>(gentree.get_axis(),height,rect,zip,start_width);
    }

}
fn draw_node<V:Vertex>(height:usize,range:Range<Numf32>,div:&DivNode<Numf32>,faafa:(Axis,compt::LevelDesc),verticies:&mut [V],width:f32){
        let (div_axis,level)=faafa;
        let line_axis=div_axis.next();

        let width=(((height-level.get_depth()) + 1) as f32)/(height as f32)*width;
        
        
        let a=div_axis;
        let b=line_axis;

        let mut p1 = axgeom::Vec2::new(0.0, 0.0);
        *p1.get_axis_mut(a) = div.divider().0.into_inner();

        *p1.get_axis_mut(b) = range.start.0.into_inner();

        let mut p2 = axgeom::Vec2::new(0.0, 0.0);
        *p2.get_axis_mut(a) = div.divider().0.into_inner();
        *p2.get_axis_mut(b) = range.end.0.into_inner();

        self::draw_line(verticies,&p1,&p2,width);
        //self::draw_line(&mut verticies[*counter*6..*counter*6+6],&p1,&p2,width);
        
    }


 fn draw_line<V:Vertex>(verticies: &mut [V], p1: &axgeom::Vec2, p2: &axgeom::Vec2, width: f32) {
    debug_assert!(verticies.len()==6);
    
    //TODO make these floating points fast approx since they just graphics.
    let (p1,p2)=(*p1,*p2);


    let offset = p2 - p1;
    let len_sqr=offset.len_sqr();
    let norm = if len_sqr > 0.0001 {
        offset / len_sqr.sqrt()
    } else {
        axgeom::Vec2::new(1.0, 0.0)
    };

    let norm90 = norm.rotate90();

    let xxx = norm90 * width;
    let yyy = norm90 * -width;
    let topleft = p1 + xxx;
    let topright = p1 + yyy;
    let bottomleft = p2 + xxx;
    let bottomright = p2 + yyy;

    let topleft=topleft.get();
    let topright=topright.get();
    let bottomleft=bottomleft.get();
    let bottomright=bottomright.get();

    unsafe {
        verticies.get_unchecked_mut(0).set_pos(*topleft.0,*topleft.1);
        verticies.get_unchecked_mut(1).set_pos(*topright.0,*topright.1);
        verticies.get_unchecked_mut(2).set_pos(*bottomleft.0,*bottomleft.1);
        verticies.get_unchecked_mut(3).set_pos(*bottomright.0,*bottomright.1);
        verticies.get_unchecked_mut(4).set_pos(*bottomleft.0,*bottomleft.1);
        verticies.get_unchecked_mut(5).set_pos(*topright.0,*topright.1);

    }

}