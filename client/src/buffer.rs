use crate::context::GameContext;
pub struct Buffer {
    items: Vec<BufferItem>,
    index: usize,
}
impl Buffer {
    pub fn new() -> Self {
        let mut s = Self {
            items: vec![],
            index: 0,
        };
        s.add("main".to_string());
        s
    }
    pub fn add(&mut self, name: String) {
        let buffer = BufferItem::new(name);
        self.items.push(buffer);
    }
    pub fn add_item(
        &mut self,
        item: String,
        buffer: String,
        speak: bool,
        ctx: &mut GameContext,
    ) -> anyhow::Result<()> {
        if speak {
            ctx.speaker.speak(&item, false)?;
        }
        let mut found = false;
        for i in self.items.iter_mut() {
            if i.name == buffer {
                found = true;
                i.add(item.clone());
            }
        }
        if !found {
            self.add(buffer.clone());
            self.add_item(item, buffer, false, ctx)?;
            return Ok(());
        }
        if self.items.len() >= 1 {
            self.items[0].add(item.clone());
        }
        Ok(())
    }
    pub fn get_item(&mut self) -> &mut BufferItem {
        &mut self.items[self.index]
    }
    pub fn speak(&self, ctx: &mut GameContext) -> anyhow::Result<()> {
        ctx.speaker.speak(
            format!(
                "{}, {} items, {} of {}",
                &self.items[self.index].name,
                self.items[self.index].items.len(),
                self.index + 1,
                self.items.len()
            ),
            false,
        )?;
        Ok(())
    }
    pub fn next(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if self.items.len() == 0 {
            return Ok(());
        }
        if self.index < self.items.len() - 1 {
            self.index += 1;
            self.speak(ctx)?;
        }
        Ok(())
    }
    pub fn previous(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if self.items.len() == 0 {
            return Ok(());
        }
        if self.index > 0 {
            self.index -= 1;
            self.speak(ctx)?;
        }
        Ok(())
    }
    pub fn first(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if self.items.len() == 0 {
            return Ok(());
        }
        self.index = 0;
        self.speak(ctx)?;
        Ok(())
    }
    pub fn last(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if self.items.len() == 0 {
            return Ok(());
        }
        self.index = self.items.len() - 1;
        self.speak(ctx)?;
        Ok(())
    }
}

pub struct BufferItem {
    name: String,
    items: Vec<String>,
    index: usize,
}
impl BufferItem {
    pub fn new(name: String) -> Self {
        Self {
            name,
            items: vec![],
            index: 0,
        }
    }
    pub fn add(&mut self, item: String) {
        self.items.push(item);
    }
    pub fn speak(&self, ctx: &mut GameContext) -> anyhow::Result<()> {
        ctx.speaker.speak(&self.items[self.index], true)?;
        Ok(())
    }
    pub fn get(&self) -> String {
        self.items[self.index].clone()
    }
    pub fn next(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if self.items.len() == 0 {
            return Ok(());
        }
        if self.index < self.items.len() - 1 {
            self.index += 1;
            self.speak(ctx)?;
        }
        Ok(())
    }
    pub fn previous(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if self.items.len() == 0 {
            return Ok(());
        }
        if self.index > 0 {
            self.index -= 1;
            self.speak(ctx)?;
        }
        Ok(())
    }
    pub fn first(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if self.items.len() == 0 {
            return Ok(());
        }
        self.index = 0;
        self.speak(ctx)?;
        Ok(())
    }
    pub fn last(&mut self, ctx: &mut GameContext) -> anyhow::Result<()> {
        if self.items.len() == 0 {
            return Ok(());
        }
        self.index = self.items.len() - 1;
        self.speak(ctx)?;
        Ok(())
    }
}
