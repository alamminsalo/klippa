use crate::*;
use geo::wkt;
use wkt::ToWkt;

#[test]
fn test_poly_diagonal() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = wkt!(POLYGON((0.2526855468749994 4.937724274302482,5.174560546875 0.0549316322096729,3.3508300781249996 -1.0436434559084802,-1.3073730468750009 4.039617826768435,0.2526855468749994 4.937724274302482)));

    let clip = rect.clip(&Geometry::Polygon(g)).unwrap();
    assert_eq!(
            clip.to_wkt().to_string(),
            "MULTIPOLYGON(((1.1979154595268593 4,4 1.2201655310741821,4 0,2.3944552358035858 0,0 2.612947796960526,0 4,1.1979154595268593 4)))"
        );
}

#[test]
fn test_poly_angle() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = wkt!(POLYGON((2.7465820312500004 4.423090477960912,2.7026367187499996 3.19536379832941,4.746093749999999 3.217302058187144,4.7900390625 1.5159363834516881,1.109619140625 1.603794430058997,1.1755371093750002 4.543570279371764,2.7465820312500004 4.423090477960912)));

    let clip = rect.clip(&Geometry::Polygon(g)).unwrap();
    println!("{}", clip.to_wkt());
    assert_eq!(
            clip.to_wkt().to_string(),
            "MULTIPOLYGON(((2.731437908719937 4,2.7026367187499996 3.19536379832941,4 3.209292103333472,4 1.5347959946504444,1.109619140625 1.603794430058997,1.1633487485863934 4,2.731437908719937 4)))"
        );
}

#[test]
fn test_poly_cross() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = wkt!(POLYGON((1.3732910156250002 4.532618393971788,2.867431640625 4.5764249358536375,2.933349609374999 2.8223442468940902,4.812011718749999 2.8113711933311407,4.822998046874999 1.537901237431484,3.021240234375 1.5488835798473986,3.0322265624999996 -0.3515602939922644,1.417236328125 -0.37353251022881295,1.3952636718749996 1.4939713066293194,-0.7690429687499999 1.482988685660274,-0.7360839843749998 2.8333171968552904,1.3293457031250002 2.7126091154394203,1.109619140625 4.4449973697272895,1.3732910156250002 4.532618393971788)));

    let clip = rect.clip(&Geometry::Polygon(g)).unwrap().to_wkt();
    println!("{clip}");
}

#[test]
fn test_poly_star() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = wkt!(POLYGON((1.9335937500000013 -1.120534032250049,1.0986328124999993 0.5932511181408557,-2.1093749999999987 0.46142079353062115,0.28564453125000056 2.482133403730572,-0.8569335937499997 5.156598738411162,1.7797851562500016 3.798483975036973,4.987792968750002 4.609278084409837,3.8452148437499996 2.2625953010152386,6.26220703125 0.9008417889908884,2.7026367187500018 0.6811362994451144,1.9335937500000013 -1.120534032250049)));

    let clip = rect.clip(&Geometry::Polygon(g)).unwrap();
    println!("{}", clip.to_wkt());

    assert_eq!(
            clip.to_wkt().to_string(),
            "MULTIPOLYGON(((1.3885508542788023 4,1.7797851562500016 3.798483975036973,2.577108391522845 4,4 4,4 2.5805006182528833,3.8452148437499996 2.2625953010152386,4 2.17538805590176,4 0.7612127721094657,2.7026367187500018 0.6811362994451144,2.411893693994855 0,1.3876664230799656 0,1.0986328124999993 0.5932511181408557,0 0.5481037466989945,0 2.241130982330577,0.28564453125000056 2.482133403730572,0 3.1507497374007207,0 4,1.3885508542788023 4)))"
        );
}

#[test]
fn test_poly_concave_1() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = wkt!(POLYGON((3.7353515625000004 4.740675384778385,3.790283203125001 2.756504385543252,0.5712890625000011 2.7784514150468738,0.5603027343750014 4.718777551249872,2.13134765625 3.1624555302378496,3.7353515625000004 4.740675384778385)));

    let clip = rect.clip(&Geometry::Polygon(g)).unwrap();
    println!("{}", clip.to_wkt());

    assert_eq!(
            clip.to_wkt().to_string(),
            "MULTIPOLYGON(((1.2858799649243433 4,2.13134765625 3.1624555302378496,2.982575447670533 4,3.755857110697265 4,3.790283203125001 2.756504385543252,0.5712890625000011 2.7784514150468738,0.5643725275296553 4,1.2858799649243433 4)))"
        );
}

#[test]
fn test_poly_concave_2() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = wkt!(POLYGON((3.735351562499999 5.189423479732426,2.065429687499999 2.6467632307409588,5.284423828124999 2.668712251961324,-0.12084960937499983 -1.417091829441631,2.6257324218750004 1.7575368113083272,-0.8349609375000002 1.691648704756986,1.8566894531249996 5.145656780300527,3.735351562499999 5.189423479732426)));

    let clip = rect.clip(&Geometry::Polygon(g)).unwrap();
    println!("{}", clip.to_wkt());

    assert_eq!(
            clip.to_wkt().to_string(),
            "MULTIPOLYGON(((2.954183772492809 4,2.065429687499999 2.6467632307409588,4 2.6599542845255755,4 1.6978262866378853,1.7538777812611581 0,1.1051706266347496 0,2.6257324218750004 1.7575368113083272,0 1.7075455177661985,0 2.763096107782738,0.96389839625035 4,2.954183772492809 4)))"
        );
}

#[test]
fn test_poly_spiral() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = wkt!(POLYGON((1.2414550781249996 4.434044005032575,1.2414550781249996 0.9337965488500259,4.39453125 0.99970513084196,4.361572265624999 4.193029605360763,2.515869140625 4.160158150193411,2.559814453125 2.910124912012904,3.62548828125 2.9430409100551316,3.6584472656249996 2.4052990502867857,2.17529296875 2.3723687086440606,2.0654296875 4.488809196778661,4.757080078125001 4.477856485570598,4.801025390624999 0.41747677467076016,0.4174804687500001 0.37353251022881295,0.5493164062499997 4.412136788910175,1.2414550781249996 4.434044005032575)));

    let clip = rect.clip(&Geometry::Polygon(g)).unwrap();
    println!("{}", clip.to_wkt());

    assert_eq!(
            clip.to_wkt().to_string(),
            "MULTIPOLYGON(((1.2414550781249996 4,1.2414550781249996 0.9337965488500259,4 0.991458265298099,4 0.4094466364566678,0.4174804687500001 0.37353251022881295,0.5358626395041394 4,1.2414550781249996 4)),((2.5214995508770635 4,2.559814453125 2.910124912012904,3.62548828125 2.9430409100551316,3.6584472656249996 2.4052990502867857,2.17529296875 2.3723687086440606,2.0908035085756933 4,2.5214995508770635 4)))"
        );
}

#[test]
fn test_poly_alternating() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = wkt!(POLYGON((0.6042480468750002 4.412136788910175,0.7031249999999996 -0.3845185979490111,1.7028808593749993 -0.34057416628374426,1.4062500000000002 4.3683204208762305,2.142333984375 4.401182938278325,2.373046875 -0.3515602939922502,3.779296875 -0.31860187370565995,3.581542968749999 4.390228926463408,4.262695312499999 4.3245014930191985,4.39453125 -0.6591651462894532,2.1313476562499996 -0.59325111814087,1.9226074218749998 4.160158150193411,1.6918945312499998 4.160158150193411,1.8786621093749998 -0.6371938961998609,0.1757812499999994 -0.5493079911125278,0.28564453124999956 4.423090477960898,0.6042480468750002 4.412136788910175)));

    let clip = rect.clip(&Geometry::Polygon(g)).unwrap();
    println!("{}", clip.to_wkt());

    assert_eq!(
            clip.to_wkt().to_string(),
            "MULTIPOLYGON(((0.6127437228557678 4,0.6951986379166918 0,0.18791800426372474 0,0.27629650565158514 4,0.6127437228557678 4)),((1.6981297104779016 4,1.853855266008407 0,1.6814268204799385 0,1.4294518791135804 4,1.6981297104779016 4)),((2.1618086479190275 4,2.355981048446358 0,2.105295748986531 0,1.9296405738552318 4,2.1618086479190275 4)),((3.765916745677536 0,3.597931175889893 4,4 4,4 0,3.765916745677536 0)))"
        );
}

#[test]
fn test_poly_complex() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = wkt!(POLYGON((-0.28564453125000056 3.743671274749744,3.7573242187499987 3.6888551431470518,3.7573242187499987 2.7235830833483874,-0.20874023437500058 2.8113711933311407,-0.20874023437500058 3.0417830279332634,3.4716796875000004 2.975955935944782,3.4716796875000004 3.2282710112526445,-0.28564453125000056 3.2282710112526445,-0.30761718749999994 2.097919733594921,3.6804199218749987 1.9991059831233287,3.669433593749999 1.680667133750731,0.5053710937499998 1.6587038068676208,0.49438476562500006 1.2633253574893217,4.482421875 1.3621763466641852,4.39453125 0.8349313860427259,0.1867675781250001 0.9557662177941495,0.16479492187499967 1.7794990011582144,1.8127441406249998 1.8344033244935218,-0.2966308593750003 1.9002862838753884,-0.1867675781250001 0.5712795966325501,3.3508300781249996 0.41747677467076016,3.482666015625 -0.5383221578577064,2.801513671875 -0.6042368463810561,2.867431640625 0.08789059053081871,-0.5712890625000001 0.23071226715249793,-0.6042480468750002 3.721745231068965,-0.28564453125000056 3.743671274749744)));

    let clip = rect.clip(&Geometry::Polygon(g)).unwrap();
    println!("{}", clip.to_wkt());

    assert_eq!(
            clip.to_wkt().to_string(),
            "MULTIPOLYGON(((4 0.8462611851788037,0.1867675781250001 0.9557662177941495,0.16479492187499967 1.7794990011582144,1.8127441406249998 1.8344033244935218,0 1.8910214927123135,0 2.0902977363133655,3.6804199218749987 1.9991059831233287,3.669433593749999 1.680667133750731,0.5053710937499998 1.6587038068676208,0.49438476562500006 1.2633253574893217,4 1.3502186145179023,4 0.8462611851788037)),((2.859060972318771 0,2.867431640625 0.08789059053081871,0 0.20698470426327326,0 0.5631595718705923,3.3508300781249996 0.41747677467076016,3.4084137812450663 0,2.859060972318771 0)),((0 3.038049551074215,3.4716796875000004 2.975955935944782,3.4716796875000004 3.2282710112526445,0 3.2282710112526445,0 3.73979839588651,3.7573242187499987 3.6888551431470518,3.7573242187499987 2.7235830833483874,0 2.806750766489943,0 3.038049551074215)))"
        );
}

#[test]
fn test_poly_partial_cover() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = wkt!(POLYGON((-0.28564453125000056 4.466903610394141,4.493408203125 4.466903610394141,4.251708984375 0.2856433479945224,3.5705566406249996 -0.17578097424708972,-0.2636718750000001 -0.241698501901638,-0.28564453125000056 4.466903610394141)));

    let clip = rect.clip(&Geometry::Polygon(g)).unwrap();
    println!("{}", clip.to_wkt());

    assert_eq!(
            clip.to_wkt().to_string(),
            "MULTIPOLYGON(((4 0.11513134934538197,3.830043661190787 0,0 0,0 4,4 4,4 0.11513134934538197)))"
        );
}

#[test]
fn test_poly_full_cover() {
    let rect = ClipRect::new(0.0, 0.0, 4.0, 4.0);
    let g = wkt!(POLYGON((-1. -1., -1. 5., 5. 5., 5. -1., -1. -1.)));

    let clip = rect.clip(&Geometry::Polygon(g)).unwrap();
    println!("{}", clip.to_wkt());

    assert_eq!(
        clip.to_wkt().to_string(),
        "MULTIPOLYGON(((0 4,4 4,4 0,0 0,0 4)))"
    );
}
