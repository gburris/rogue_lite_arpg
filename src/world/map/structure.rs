use crate::world::map::tiles::TileType;

pub struct Structure {
    floor_type: TileType,
    ascii_layout: &'static str,
}

const HUB_TOWN: Structure = Structure {
    floor_type: TileType::Grass,
    ascii_layout: "
        #########################
        #########################
        ##ccccccccccccccccccccc##
        ##ccccccccccccccccccccc##
        ##ccccccccccccccccccccc##
        ##ccccccccccccccccccccc##
        ##ccccccccccccccccccccc##
        ##ccccccccccccccccccccc##
        ##ccccccccccccccccccccc##
        ##ccccccccccccccccccccc##
        ##ccccccccccccccccccccc##
        ##ccccccccccccccccccccc##
        ##ccccccccccccccccccccc##
        ##ccccccccccccccccccccc##
        ##ccccccccccccccccccccc##
        ##ccccccccccccccccccccc##
        ##ccccccccccccccccccccc##
        ##ccccccccccccccccccccc##
        ##ccccccccccccccccccccc##
        ##ccccccccccccccccccccc##
        ##ccccccccwwwwwcccccccc##
        ##ccccccccwwwwwcccccccc##
        ##ccccccccwwwwwcccccccc##
        ##########wwwww##########
        ##########wwwww##########
                  wwwww
                  wwwww                   
                  wwwww    
                  wwwww
                  wwwww
        ",
};
