/// Iterator over games in a SCID database
///
/// This iterator lazily parses games on demand, minimizing memory usage.
pub struct GameIterator<'a> {
    reader: &'a ScidReader,
    index: usize,
}

impl<'a> Iterator for GameIterator<'a> {
    type Item = Result<Game>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.reader.game_count() {
            None
        } else {
            let game = self.reader.game(self.index);
            self.index += 1;
            Some(game)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.reader.game_count() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for GameIterator<'a> {
    fn len(&self) -> usize {
        self.reader.game_count() - self.index
    }
}
