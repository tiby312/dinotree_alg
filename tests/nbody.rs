
fn test_nodemass(){
    let mut b1=create_bots_f64(|_id,pos|Bot{pos,vel:[0.0;2],force:[0.0;2],mass:BOT_MASS},&[0,100,0,100],50,[2,20]);
    let mut b2=create_bots_f64(|_id,pos|Bot{pos,vel:[0.0;2],force:[0.0;2],mass:BOT_MASS},&[800,900,800,900],50,[2,20]);

    let control={
        for i in b1.iter_mut(){
            for j in b2.iter_mut(){
                NodeMass::handle_bot(i,j);
            }
        }

        let control:Vec<[f64;2]> =b1.iter().chain(b2.iter()).map(|a|a.val.force).collect();
        for b in b1.iter_mut().chain(b2.iter_mut()){
            b.val.force=[0.0;2]
        }
        control
    };


    let test={
        let mut n1=NodeMass::new(b1.iter(),b1.len());
        let mut n2=NodeMass::new(b2.iter(),b2.len());

        n1.handle_with(&mut n2);
        

        let b1len=b1.len();
        let b2len=b2.len();
        n1.undo(b1.iter_mut(),b1len);
        n2.undo(b2.iter_mut(),b2len);

        let test:Vec<[f64;2]>=b1.iter().chain(b2.iter()).map(|a|a.val.force).collect();
        for b in b1.iter_mut().chain(b2.iter_mut()){
            b.val.force=[0.0;2]
        }
        test
    };

    for (a,b) in control.iter().zip(test.iter()){
        let diffx=(a[0]-b[0]).abs();
        let diffy=(a[1]-b[1]).abs();
        println!("diff={:?}",(diffx,diffy));
    }

    //one list of bots.
    //second list of bots.

    //handle as node masses



}
