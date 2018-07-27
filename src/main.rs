#![feature(use_extern_macros)]

extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;

extern crate treexml;

mod database;
use database::Database;

use gtk::prelude::*;
use gtk::{
    Builder, ButtonsType, DialogFlags, FileChooserAction, FileChooserDialog, Label, MenuItem,
    MessageDialog, MessageType, TreeStore, TreeView, TreeViewColumn, Window,
};

use relm::{Relm, Update, Widget};

use std::path::Path;

fn update_treestore(db: &mut Database, input: &TreeStore) {
    input.clear();
    for artist in &db.entries {
        let iter = input.insert_with_values(None, None, &[0], &[&artist.name]);

        for album in &artist.albums {
            let iter = input.insert_with_values(Some(&iter), None, &[0], &[&album.title]);

            for track in &album.tracks {
                input.insert_with_values(
                    Some(&iter),
                    None,
                    &[0, 1],
                    &[&track.title, &track.lyrics],
                );
            }
        }
    }
    db.clean();
}

#[derive(Msg)]
pub enum Msg {
    SelectedItem,
    MenuOpen,
    AddArtist,
    AddAlbum,
    AddTrack,
    Quit,
}

pub struct Model {
    db: Database,
    tree_store: gtk::TreeStore,
}

struct Win {
    tree_view: TreeView,
    model: Model,
    window: Window,
    text_viewer: Label,
}

impl Update for Win {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    //Return empty model
    fn model(_: &Relm<Self>, _: ()) -> Model {
        Model {
            db: Database::empty(),
            tree_store: TreeStore::new(&[
                String::static_type(),
                String::static_type(),
                i32::static_type(),
            ]),
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::SelectedItem => {
                let selection = self.tree_view.get_selection();
                if let Some((model, iter)) = selection.get_selected() {
                    let mut path = model.get_path(&iter).expect("failed to get path");

                    if path.get_depth() != 3 {
                        return;
                    }

                    //TODO this leaks memory, fix
                    if let Some(lyrics) = model.get_value(&iter, 1).get::<String>() {
                        self.text_viewer.set_text(&lyrics);
                    } else {
                        self.text_viewer.set_text("");
                    }
                }
            }
            Msg::MenuOpen => {
                let dialog = FileChooserDialog::new(
                    Some("Open..."),
                    Some(&self.window),
                    FileChooserAction::Open,
                );
                dialog.add_button("Open", 0);
                dialog.add_button("Close", 1);
                let result = dialog.run();
                if result == 0 {
                    let filename = dialog.get_filename().expect("Failed to get filename");
                    let file = Path::new(&filename);
                    if !file.exists() {
                        let dialog = MessageDialog::new(
                            Some(&self.window),
                            DialogFlags::all(),
                            MessageType::Error,
                            ButtonsType::None,
                            format!("File {} does not exist!", file.to_string_lossy()).as_str(),
                        );
                        dialog.run();
                    } else {
                        self.model.db = Database::from(file.to_str().unwrap()).unwrap();
                        self.model.db.save("").unwrap();
                        update_treestore(&mut self.model.db, &self.model.tree_store);
                    }
                }
                dialog.destroy();
            }
            Msg::AddArtist => println!("todo too"),
            Msg::AddAlbum => println!("todo"),
            Msg::AddTrack => {
                let selection = self.tree_view.get_selection();
                if let Some((_, iter)) = selection.get_selected() {
                    let model = self.model.tree_store.clone();
                    //TODO apparently paths need to be freed manually?
                    let path = model.get_path(&iter).expect("Failed to get path");
                    if path.get_depth() < 3 {
                        return;
                    }
                    let iter = model.iter_parent(&iter).unwrap();
                    model.insert_with_values(
                        Some(&iter),
                        None,
                        &[0, 1],
                        &[&String::new(), &String::new()],
                    );
                    // let iter = model
                    //     .iter_nth_child(Some(&iter), model.iter_n_children(Some(&iter)))
                    //     .unwrap();
                }
            }
            Msg::Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Win {
    type Root = Window;
    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let glade_src = include_str!("window.glade");
        let builder = Builder::new_from_string(glade_src);

        //Load glade items
        let window: Window = builder.get_object("window").unwrap();
        let open: MenuItem = builder.get_object("menu_open").unwrap();
        let text_viewer = builder.get_object("text_viewer").unwrap();
        let tree_view: TreeView = builder.get_object("tree_view").unwrap();
        let col_name: TreeViewColumn = builder.get_object("view_column").unwrap();
        let col_lyrics: TreeViewColumn = builder.get_object("lyric_column").unwrap();
        let add_menu_artist: MenuItem = builder.get_object("add_menu_artist").unwrap();
        let add_menu_album: MenuItem = builder.get_object("add_menu_album").unwrap();
        let add_menu_track: MenuItem = builder.get_object("add_menu_track").unwrap();

        //Setup tree view
        let cell_name = gtk::CellRendererText::new();
        col_name.pack_start(&cell_name, true);
        col_name.add_attribute(&cell_name, "text", 0);

        let cell_lyrics = gtk::CellRendererText::new();
        col_lyrics.pack_start(&cell_lyrics, true);
        col_lyrics.add_attribute(&cell_lyrics, "text", 0);

        cell_name
            .set_property("editable", &true)
            .expect("failed to set editable");
        tree_view.set_model(Some(&model.tree_store));

        window.show_all();

        connect!(
            relm,
            window,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );
        connect!(
            relm,
            tree_view,
            connect_cursor_changed(_),
            Msg::SelectedItem
        );
        connect!(relm, open, connect_activate(_), Msg::MenuOpen);
        connect!(relm, add_menu_artist, connect_activate(_), Msg::AddArtist);
        connect!(relm, add_menu_album, connect_activate(_), Msg::AddAlbum);
        connect!(relm, add_menu_track, connect_activate(_), Msg::AddTrack);

        let model1 = model.tree_store.clone();
        cell_name.connect_edited(move |_, path, string| {
            let iter = model1.get_iter(&path).unwrap();
            model1.set(&iter, &[0], &[&string.to_owned()]);
        });

        Win {
            model,
            tree_view,
            window,
            text_viewer,
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}
