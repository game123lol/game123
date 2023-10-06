#![cfg(test)]

use crate::map::{Map, WorldMap};
#[test]
fn map_index() {
    let ch_idx = WorldMap::xy_index_chunk(0, 0);
    assert_eq!(ch_idx, 112);
    let ch_idx = WorldMap::xy_index_chunk(15, 15);
    assert_eq!(ch_idx, 112);
    let ch_idx = WorldMap::xy_index_chunk(-15, -15);
    assert_eq!(ch_idx, 112);
    let ch_idx = WorldMap::xy_index_chunk(-1, -1);
    assert_eq!(ch_idx, 96);
}
