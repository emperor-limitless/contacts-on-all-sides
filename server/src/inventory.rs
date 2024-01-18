use serde_derive::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct InventoryItem {
    name: String,
    count: isize,
}
#[derive(Default, Serialize, Deserialize)]
pub struct Inventory {
    items: Vec<InventoryItem>,
    index: usize,
}
impl Inventory {
    pub fn cycle(&mut self, direction: usize) {
        if direction == 0 {
            if self.index > 0 {
                self.index -= 1;
            } else {
                self.index = self.items.len() - 1;
            }
        } else if direction == 1 {
            if self.index < self.items.len() - 1 {
                self.index += 1;
            } else {
                self.index = 0;
            }
        }
    }
    pub fn get_text(&self) -> String {
        if let Some(item) = self.get(self.item()) {
            return format!(
                "{}:  You have {}, {} of {}",
                item.name,
                item.count,
                self.index + 1,
                self.items.len()
            );
        }
        String::from("Error")
    }
    pub fn item(&self) -> &str {
        &self.items[self.index].name
    }
    pub fn get(&self, name: &str) -> Option<&InventoryItem> {
        for i in &self.items {
            if i.name == name {
                return Some(i);
            }
        }
        None
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn give(&mut self, item_name: &str, amount: isize) {
        for i in 0..self.items.len() {
            if self.items[i].name == item_name {
                self.items[i].count += amount;
                if self.items[i].count <= 0 {
                    self.items.remove(i);
                    if self.index != 0 {
                        self.index -= 1;
                    }
                }
                return;
            }
        }
        let item = InventoryItem {
            name: item_name.to_string(),
            count: amount,
        };
        self.items.push(item);
    }
}
