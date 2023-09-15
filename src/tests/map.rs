#[cfg(test)]
mod map {
    use crate::map::Map;
    #[test]
    fn map_index() {
        let ch_idx = Map::xy_index_chunk(0, 0);
        assert_eq!(ch_idx, 112);
        let ch_idx = Map::xy_index_chunk(15, 15);
        assert_eq!(ch_idx, 112);
        let ch_idx = Map::xy_index_chunk(-15, -15);
        assert_eq!(ch_idx, 112);
        let ch_idx = Map::xy_index_chunk(0, -8);
        assert_eq!(ch_idx, 112);
    }
}
