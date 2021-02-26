use rand::seq::SliceRandom;
static SIZE_RANGE: std::ops::Range<usize> = 5..200;
static MINE_COUNT_RANGE: std::ops::Range<usize> = 1..100;

#[derive(Debug, PartialEq)]
pub enum BlockType {
    Mine,
    Space,
    Tip(usize),
}
#[derive(Debug, PartialEq)]
pub enum BlockStatus {
    Shown,
    Hidden,
    QuestionMarked,
    Flaged,
}
#[derive(Debug, Copy, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}
#[derive(Debug)]
pub struct MineBlock {
    pub btype: BlockType,
    pub bstatus: BlockStatus,
    pub pos: Position,
}
pub struct MinePlayground {
    shown_count: usize,
    safety_block_count: usize,
    width: usize,
    height: usize,
    pub map: Vec<Vec<MineBlock>>,
}
#[derive(Debug)]
pub enum ClickResult {
    Wasted,
    NothingHappened,
    Win,
}
impl Default for MineBlock {
    fn default() -> MineBlock {
        MineBlock {
            bstatus: BlockStatus::Hidden,
            btype: BlockType::Space,
            pos: Position { x: 0, y: 0},
        }
    }
}
impl BlockType {
    fn increase (&mut self) {
        *self = match *self {
            Self::Tip(val) => Self::Tip(val + 1),
            Self::Space => Self::Tip(1),
            Self::Mine => Self::Mine,
        }
    }
}
impl MineBlock {
    fn add_tip(&mut self) {
        self.btype.increase();
    }
}

impl MinePlayground {
    fn get(&self, x: isize, y: isize) -> Result<&MineBlock, String> {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return Err("out".to_string());
        }
        Ok(&self.map[y as usize][x as usize])
    }
    pub fn init(&width: &usize, &height: &usize, &mine_count: &usize) -> Result<MinePlayground, String> {
        if !SIZE_RANGE.contains(&width) || !SIZE_RANGE.contains(&height) || !MINE_COUNT_RANGE.contains(&mine_count) {
            return Err(String::from("Parameters not in specific range!"));
        }
        let seeds_length = width * height;
        let mut mine_seeds: Vec<bool> = Vec::with_capacity(seeds_length.into());
        for i in 0..seeds_length {
            if i < mine_count { mine_seeds.push(true); }
            else { mine_seeds.push(false); }
        }
        let mut rng = rand::thread_rng();
        mine_seeds.shuffle(&mut rng);
        let mut mine_map: Vec<Vec<MineBlock>> = vec![];
        for i in 0..height {
            mine_map.push(mine_seeds[i * width..i * width + width].iter().enumerate().map(|(j, &is_mine_block)| {
                MineBlock {
                    btype: if is_mine_block { BlockType::Mine } else { BlockType::Space },
                    pos: Position { x: j, y: i },
                    ..Default::default()
                }
            }).collect());
        }
        for y in 0..mine_map.len() {
            let row = &mine_map[y];
            for x in 0..row.len() {
                if let BlockType::Space = mine_map[y][x].btype {
                    let surroundings = get_surroundings(&x, &y, &width, &height);
                    for (cur_x, cur_y) in surroundings.iter() {
                        if let BlockType::Mine = mine_map[*cur_y][*cur_x].btype {
                            mine_map[y][x].add_tip();
                        }
                    }
                }
            }
        }
        // println!("{:?}", mine_map);

        Ok(MinePlayground {
            shown_count: 0,
            safety_block_count: height * width - mine_count,
            width,
            height,
            map: mine_map,
        })
    }
    pub fn click(&mut self, x: &usize, y: &usize) -> ClickResult {
        let mut block = &mut self.map[*y][*x];
        if let BlockStatus::Hidden = block.bstatus {
            match block.btype {
                BlockType::Mine => {
                    // game over
                    for y in 0..self.height {
                        for x in 0..self.width {
                            self.map[y][x].bstatus = BlockStatus::Shown;
                        }
                    }
                    return ClickResult::Wasted;
                },
                BlockType::Tip(_) => {
                    block.bstatus = BlockStatus::Shown;
                    self.shown_count += 1;
                },
                BlockType::Space => {
                    block.bstatus = BlockStatus::Shown;
                    let surroundings = get_surroundings(x, y, &self.width, &self.height);
                    self.shown_count += 1;
                    for (cur_x, cur_y) in surroundings.iter() {
                        self.click(cur_x, cur_y);
                    }
                }
            }
            if self.shown_count == self.safety_block_count {
                return ClickResult::Win;
            }
        }
        ClickResult::NothingHappened
    }
    pub fn right_click(&mut self, x: &usize, y: &usize) {
        let mut block = &mut self.map[*y][*x];
        if let BlockStatus::Shown = block.bstatus {
            return;
        }
        match block.bstatus {
            BlockStatus::Hidden => { block.bstatus = BlockStatus::Flaged; }
            BlockStatus::Flaged => { block.bstatus = BlockStatus::QuestionMarked; }
            BlockStatus::QuestionMarked => { block.bstatus = BlockStatus::Hidden; }
            _ => {}
        }
    }
}

fn get_surroundings(&x: &usize, &y: &usize, &max_width: &usize, &max_height: &usize) -> Vec<(usize, usize)> {
    let max_x = max_width - 1;
    let max_y = max_height - 1;
    let mut r = vec![];
    if x > 0 { r.push((x - 1, y)); }
    if x < max_x { r.push((x + 1, y)); }
    if y > 0 {
        r.push((x, y - 1));
        if x > 0 { r.push((x - 1, y - 1)); }
        if x < max_x { r.push((x + 1, y - 1)); }
    }
    if y < max_y {
        r.push((x, y + 1));
        if x > 0 { r.push((x - 1, y + 1)); }
        if x < max_x { r.push((x + 1, y + 1)); }
    }
    r
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_init_map() {
        assert!(MinePlayground::init(&0, &0, &0).is_err());
        assert!(MinePlayground::init(&8, &8, &10).is_ok());
    }
    #[test]
    fn test_get_surroundings() {
        assert_eq!(get_surroundings(&9, &9, &10, &10), vec![(8, 9), (9, 8), (8, 8)]);
    }
}