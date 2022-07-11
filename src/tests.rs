// Copyright (c) 2018-2022, agnos.ai UK Ltd, all rights reserved.
//---------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::{Error, Graph, RoleCreds, Server};

    #[test]
    fn load_rdfox() -> Result<(), Error> {
        env_logger::init();
        let role_creds = RoleCreds::default();
        let server = Server::start(&role_creds)?;

        let connection = server.connection(&role_creds)?;

        assert!(connection.get_number_of_threads()? > 0);

        // We next specify how many threads the server should use during import of data and reasoning.
        connection.set_number_of_threads(2)?;

        connection.create_data_store("example")?;

        let mut ds_connection = connection.connect_to_data_store("example")?;

        let test_graph = Graph { local_name: "test".to_string() };
        ds_connection.import_data_from_file("test.ttl", &test_graph)?;

        let count = ds_connection.get_triples_count();
        assert!(count.is_ok());
        assert_eq!(count.unwrap(), 8);

        Ok(())
    }
}
