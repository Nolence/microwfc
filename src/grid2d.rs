use std::fmt::Debug;

use rand::Rng;

use crate::{
    grid::SizeErr, pixel::PixelChangeResult, Grid, ImplementedGrid, Pixel, PossibleValues,
};

pub type Vec2i = (usize, usize);

impl<T: PossibleValues + Debug> Grid<Vec2i, Vec<Vec<Pixel<T>>>> {
    fn update<F: Fn(&Self, Vec2i, &T) -> bool>(
        &mut self,
        to_update: &mut Vec<(Vec2i, Pixel<T>)>,
        ((x, y), mut pixel): (Vec2i, Pixel<T>),
        test: &F,
        effect_distance: usize,
        rng: &mut impl Rng,
        should_collapse: bool,
    ) -> PixelChangeResult {
        let result = pixel.recalc(
            self,
            (x, y),
            &test,
            if should_collapse { Some(rng) } else { None },
        );
        match result {
            PixelChangeResult::Invalid => {
                return PixelChangeResult::Invalid;
            }
            PixelChangeResult::Updated => {
                self.data[x][y] = pixel;
                for iy in 0..=(effect_distance * 2) {
                    for ix in 0..=(effect_distance * 2) {
                        let loc = (
                            ix as i128 - effect_distance as i128 + x as i128,
                            iy as i128 - effect_distance as i128 + y as i128,
                        );
                        if let Some(loc) = self.check_loc(loc) {
                            to_update.push((loc, self.data[loc.0][loc.1].clone()));
                        }
                    }
                }
            }
            PixelChangeResult::Unchanged => return result,
        }
        result
    }
}

impl<T: PossibleValues + Debug> ImplementedGrid<Vec2i, T, (i128, i128)>
    for Grid<Vec2i, Vec<Vec<Pixel<T>>>>
{
    fn new(size: Vec2i) -> Result<Self, SizeErr> {
        if size.0 == 0 || size.1 == 0 {
            return Err(SizeErr::SizeMustNotBeZero);
        }
        Ok(Self {
            size,
            data: vec![vec![Pixel::default(); size.1]; size.0],
        })
    }

    fn get_item(&self, location: Vec2i) -> Pixel<T> {
        self.data[location.0][location.1].clone()
    }

    fn set_item(&mut self, location: Vec2i, item: Pixel<T>) {
        self.data[location.0][location.1] = item;
    }

    fn unidirectional_neighbors(&self, location: Vec2i) -> Vec<Pixel<T>> {
        let mut v = Vec::new();
        if location.0 > 0 {
            v.push(self.get_item((location.0 - 1, location.1)));
        }
        if location.1 > 0 {
            v.push(self.get_item((location.0, location.1 - 1)));
        }
        if location.1 < self.size.1 - 1 {
            v.push(self.get_item((location.0, location.1 + 1)));
        }
        if location.0 < self.size.0 - 1 {
            v.push(self.get_item((location.0 + 1, location.1)));
        }
        v
    }

    fn neighbors(&self, location: Vec2i) -> Vec<Pixel<T>> {
        let mut v = Vec::new();
        for y in -1i128..=1 {
            for x in -1i128..=1 {
                let location = (location.0 as i128 + x, location.1 as i128 + y);
                if location.0 < self.size.0 as i128
                    && location.1 < self.size.1 as i128
                    && x >= 0
                    && y >= 0
                {
                    v.push(self.get_item((location.0 as usize, location.1 as usize)));
                }
            }
        }
        v
    }

    fn check_loc(&self, location: (i128, i128)) -> Option<Vec2i> {
        if location.0 < 0
            || location.1 < 0
            || location.0 >= self.size.0 as i128
            || location.1 >= self.size.1 as i128
        {
            None
        } else {
            Some((location.0 as usize, location.1 as usize))
        }
    }

    fn wfc<F, R>(&mut self, test: F, effect_distance: usize, rng: &mut R) -> bool
    where
        F: Fn(&Self, Vec2i, &T) -> bool,
        R: Rng,
    {
        {
            let mut data = self.data.clone();
            for (y, xv) in data.iter_mut().enumerate() {
                for (x, pixel) in xv.iter_mut().enumerate() {
                    if let PixelChangeResult::Invalid =
                        pixel.recalc(self, (x, y), &test, None::<&mut R>)
                    {
                        return false;
                    }
                }
            }
            self.data = data;
        }
        loop {
            let mut done = true;
            let backup = self.data.clone();

            let mut updatable = Vec::new();
            for (x, xv) in self.data.iter().enumerate() {
                for (y, pixel) in xv.iter().enumerate() {
                    updatable.push(((x, y), pixel.clone()));
                }
            }
            let mut to_update = updatable
                .into_iter()
                .filter(|x| x.1.determined_value.is_none())
                .collect::<Vec<_>>();
            to_update.sort_by(|a, b| a.1.possible_values.len().cmp(&b.1.possible_values.len()));
            if to_update.is_empty() {
                break;
            }
            to_update = vec![to_update.remove(0)];
            let mut i = 0;
            while !to_update.is_empty() {
                let item = to_update.remove(0);
                let r = self.update(&mut to_update, item, &test, effect_distance, rng, i == 0);
                match r {
                    PixelChangeResult::Unchanged => (),
                    PixelChangeResult::Updated => done = false,
                    PixelChangeResult::Invalid => {
                        done = false;
                        self.data = backup;
                        break;
                    }
                }
                i += 1;
            }

            if done {
                break;
            }
        }
        true
    }
}
