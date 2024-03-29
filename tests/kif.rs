use shogi::{alloc::PositionMovesAllocator, Position};
use tsumeshogi_check::shogi;
use tsumeshogi_check::shogi::moves;

#[test]
fn kif_moves() {
  let mut pos = Position::default();
  let mut allocator = PositionMovesAllocator::default();
  let mut last_move: Option<moves::Move> = None;
  for s in vec![
    "２六歩(27)",
    "３四歩(33)",
    "７六歩(77)",
    "４四歩(43)",
    "４八銀(39)",
    "４二飛(82)",
    "５八金(49)",
    "６二玉(51)",
    "５六歩(57)",
    "７二玉(62)",
    "６八玉(59)",
    "８二玉(72)",
    "７八玉(68)",
    "７二銀(71)",
    "２五歩(26)",
    "３三角(22)",
    "３六歩(37)",
    "３二銀(31)",
    "９六歩(97)",
    "９四歩(93)",
    "６八銀(79)",
    "５二金(41)",
    "５七銀(68)",
    "４三銀(32)",
    "１六歩(17)",
    "１四歩(13)",
    "３五歩(36)",
    "同　歩(34)",
    "４六銀(57)",
    "３四銀(43)",
    "３八飛(28)",
    "４五歩(44)",
    "３三角成(88)",
    "同　桂(21)",
    "７七角打",
    "４三金(52)",
    "３五銀(46)",
    "同　銀(34)",
    "同　飛(38)",
    "３四歩打",
    "３六飛(35)",
    "２八角打",
    "２四歩(25)",
    "同　歩(23)",
    "２六飛(36)",
    "１九角成(28)",
    "２四飛(26)",
    "４六歩(45)",
    "同　歩(47)",
    "同　馬(19)",
    "２一飛成(24)",
    "５六馬(46)",
    "５七歩打",
    "６五馬(56)",
    "１一龍(21)",
    "７六馬(65)",
    "４四歩打",
    "同　金(43)",
    "３一龍(11)",
    "３二飛(42)",
    "４一龍(31)",
    "８四香打",
    "８八香打",
    "２八歩打",
    "３七桂(29)",
    "５二銀打",
    "４四龍(41)",
    "４三銀(52)",
    "４六龍(44)",
    "４四歩打",
    "７六龍(46)",
  ] {
    let m = pos.parse_kif_move(&mut allocator, s, last_move).unwrap();
    pos.do_move(&m);
    assert!(pos.is_legal());
    last_move = Some(m);
  }
}
