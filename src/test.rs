(&location_data, &dirt_data).join()
    //if there's an entity above the dirt tile we won't let the dirt tile
    //spawn a tree; nobody gets sodomized by trees on this christian minecraft server
    .map(|(dirt_location, _)| dirt_location)
    .fiter(|dirt_location| {
        (&location_data)
            .join()
            .find(|entity_location| {
                entity_location.x == dirt_location.x
                    && entity_location.y == dirt_location.y
                    && entity_location.z == dirt_location.z + I32F32::from(1)
            })
            .is_none()
    }
    // Skip if any other tree exists in a radius of 3
    .filter(|dirt_location| {
        (&location_data, &tree_data).join()
            .all(|(tree_loc, _)| {
                (tree_location.x - dirt_location.x).abs() <= 3 &&
                    (tree_location.y - dirt_location.y).abs() <= 3 &&
                    (tree_location.z - dirt_location.z).abs() <= 3 
            })
        }
    })
    //now collect and iter because iterators are lazy
    .collect::<Vec<_>>()
.iter()
    //make a tree for each remaining thing
    .for_each(|dirt_location| {
        let tree_entity = entities.create();
        tree_data.insert(tree_entity, Tree::new());
        location_data.insert( // Mutable borrow here
            tree_entity,
            WorldLocation {
                x: dirt_location.x,
                y: dirt_location.y,
                z: dirt_location.z + I32F32::from(1),
                is_walkable: false,
                texture_atlas_index: 2,
            },
        );
    });
