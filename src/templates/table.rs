use gobble::*;
use std::fmt::Write;

pub fn write_row(target: &mut String, row: &[String], head: bool) {
    write!(target, "<tr>").ok();
    let es = match head {
        true => "th",
        false => "td",
    };
    for r in row {
        write!(target, "<{1}>{}</{1}>", r, es).ok();
    }
    write!(target, "</tr>\n").ok();
}

parser! {(Table->String)
    star_until_ig(Row,ws_(EOI)).map(|v|{
        let mut res = String::new();
        let mut curr:Option<Vec<String>> = None;
        let mut is_head = false;

        for (hd,cont,row) in v{
            if !cont {
                if let Some(c) = curr.take(){
                    write_row(&mut res,&c, is_head);
                }
                is_head = false;
            }
            is_head |= hd;
            let r_len =row.len();
            if r_len <= 3{
                continue;
            }

            match curr {
                Some(ref mut v)=>{
                    for (n,s) in row[1..r_len-1].iter().enumerate(){
                        if n >v.len() {
                            v.push(s.to_string());
                        }else{
                            write!(v[n]," {}",s).ok();
                        }
                    }
                }
                None=>{
                    curr = Some(row[1..r_len-1].iter().map(|v|v.trim().to_string()).collect());
                }
            }
        }
        if let Some(c) = curr{
            write_row(&mut res,&c,is_head);
        }
        res
    })
}

parser! {(Row->(bool,bool,Vec<String>))
    (exists('*'),exists('-'), sep_until_ig(not("\n|").star(),"|",Fin))
}

parser! {(Fin->())
    ws_(or_ig!(
        "\n",
        EOI
    ))
}
