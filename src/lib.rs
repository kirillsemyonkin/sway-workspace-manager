use swayipc::Connection;

mod command;
mod workspaces;

pub use command::{Command, Position};
pub use workspaces::{OrderedWorkspaces, Workspaces};

pub fn run(
    connection: &mut Connection,
    mut workspaces: OrderedWorkspaces,
    command: Command,
) -> Result<(), swayipc::Error> {
    let num_workspaces = workspaces.names().len() - 1;

    match command {
        Command::Reorder => (),

        Command::Switch { target, carry } => {
            let target_index = target.num_existing(workspaces.current_index(), num_workspaces)?;
            let target_name = workspaces.name(target_index);

            if carry {
                // carrying out of an empty workspace silently fails
                connection.run_command(format!(
                    "move container to workspace \"{target_index}{target_name}\""
                ))?;
            }

            connection.run_command(format!("workspace \"{target_index}{target_name}\""))?;

            Workspaces::get(connection)?.reorder(connection)?;
        }

        Command::Create { target, carry } => {
            let target_index = target.num_new(workspaces.current_index(), num_workspaces)?;
            let target_name = workspaces.name(target_index).to_string();

            workspaces.insert(connection, target_index)?;

            if carry {
                // carrying out of an empty workspace silently fails
                connection.run_command(format!(
                    "move container to workspace \"{target_index}{target_name}\""
                ))?;
            }

            connection.run_command(format!("workspace \"{target_index}\""))?;

            Workspaces::get(connection)?.reorder(connection)?;
        }

        Command::Swap { target } => {
            let current_index = workspaces.current_index();
            let current_name = &workspaces.names()[current_index].as_ref().unwrap();

            let target_index = target.num_existing(workspaces.current_index(), num_workspaces)?;
            let target_name = &workspaces.names()[target_index].as_ref().unwrap();

            connection.run_command(format!(
                "rename workspace \"{target_index}{target_name}\" to a"
            ))?;
            connection.run_command(format!(
                "rename workspace \"{current_index}{current_name}\" to \"{target_index}{current_name}\""
            ))?;
            connection.run_command(format!(
                "rename workspace a to \"{current_index}{target_name}\""
            ))?;
        }

        Command::Rename { new_name } => {
            let current_index = workspaces.current_index();
            let current_name = &workspaces.names()[current_index].as_ref().unwrap();

            if !new_name.is_empty() {
                connection.run_command(format!(
                    "rename workspace \"{current_index}{current_name}\" to \"{current_index}:{new_name}\""
                ))?;
            } else {
                connection.run_command(format!(
                    "rename workspace \"{current_index}{current_name}\" to \"{current_index}\""
                ))?;
            }
        }
    }

    Ok(())
}
