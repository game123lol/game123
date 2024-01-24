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

#[test]
fn chunk_xy() {
    let ch_xy = WorldMap::xy_chunk(0, 0);
    assert_eq!(0, 0);
    let ch_xy = WorldMap::xy_chunk(15, 15);
    assert_eq!(1, 1);
    let ch_xy = WorldMap::xy_chunk(-15, -15);
    assert_eq!(-1, -1);
}
