use super::types::{Item, StationKind};

/// What `station` turns `item` into, or `None` if it has no use for it.
pub fn transform(station: StationKind, item: Item) -> Option<Item> {
    match (station, item) {
        (StationKind::ChoppingBoard, Item::Tomato) => Some(Item::ChoppedTomato),
        (StationKind::Stove, Item::ChoppedTomato) => Some(Item::TomatoSoup),
        (StationKind::Stove, Item::Sandwich) => Some(Item::Toastie),
        _ => None,
    }
}

/// What two items make when one is pushed into the other, or `None` if they
/// do not go together. The order does not matter.
pub fn combine(a: Item, b: Item) -> Option<Item> {
    match (a, b) {
        (Item::Bread, Item::Cheese) | (Item::Cheese, Item::Bread) => Some(Item::Sandwich),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chopping_board_chops_a_tomato() {
        assert_eq!(
            transform(StationKind::ChoppingBoard, Item::Tomato),
            Some(Item::ChoppedTomato)
        );
    }

    #[test]
    fn stove_cooks_chopped_tomato_into_soup() {
        assert_eq!(
            transform(StationKind::Stove, Item::ChoppedTomato),
            Some(Item::TomatoSoup)
        );
    }

    #[test]
    fn stove_toasts_a_sandwich() {
        assert_eq!(transform(StationKind::Stove, Item::Sandwich), Some(Item::Toastie));
    }

    #[test]
    fn a_station_has_no_use_for_other_items() {
        assert_eq!(transform(StationKind::Stove, Item::Tomato), None);
        assert_eq!(transform(StationKind::ChoppingBoard, Item::Bread), None);
    }

    #[test]
    fn bread_and_cheese_make_a_sandwich() {
        assert_eq!(combine(Item::Bread, Item::Cheese), Some(Item::Sandwich));
    }

    #[test]
    fn combining_works_in_either_order() {
        assert_eq!(combine(Item::Cheese, Item::Bread), Some(Item::Sandwich));
    }

    #[test]
    fn items_without_a_recipe_do_not_combine() {
        assert_eq!(combine(Item::Tomato, Item::Bread), None);
        assert_eq!(combine(Item::Bread, Item::Bread), None);
    }
}
