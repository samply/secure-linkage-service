#[cfg(test)]
mod tests {
    use secure_linkage_service::domain::*;
    use secure_linkage_service::transformations::*;
    use std::collections::HashMap;
    use std::rc::Rc;

    #[test]
    fn compute_linkage_should() {
        // Create shared record IDs
        let server_record_ids = Rc::new([
            RecordId::new("server-record-1"),
            RecordId::new("server-record-2"),
            RecordId::new("server-record-3"),
        ]);

        // Create immutable matches HashMap
        let matches = {
            let mut map = HashMap::new();
            map.insert(
                server_record_ids[0].clone(),
                (TentativeMatch(true), ConfirmedMatch(false)),
            );
            map.insert(
                server_record_ids[1].clone(),
                (TentativeMatch(false), ConfirmedMatch(false)),
            );
            Rc::new(map)
        };

        // Create shared LinkageShare
        let linkage_share = Rc::new(LinkageShare::new(
            matches.clone(),
            Some(&client_record_ids[0]),
        ));

        // Create shared client and server IDs
        let client_id = Rc::new(HostId::new("host-c"));
        let server_id = Rc::new(HostId::new("host-s"));
        let shared_key = Rc::new(Key::new("some-key"));

        // Create shared EpiLinker instances
        let client = Rc::new(EpiLinker {
            uid: HostId(client_id.0.clone()),
            key: Key(shared_key.0.clone()),
            padding: Padding(15),
            linkage_data: Some(LinkageData::ClientLinkageData),
        });

        let server = Rc::new(EpiLinker {
            uid: HostId(server_id.0.clone()),
            key: Key(shared_key.0.clone()),
            padding: Padding(15),
            linkage_data: Some(LinkageData::ServerLinkageData),
        });

        let timeout = TimeOut(10);

        // Create shared computation
        let computations = Rc::new([Computation::new(
            client.uid.clone(),
            server.uid.clone(),
            Some((*linkage_share).clone()),
        )]);

        // Create shared LinkageState
        let state = LinkageState {
            epilinkers: Rc::new([(*client).clone(), (*server).clone()]),
            computations: computations.clone(),
        };

        // Expected result
        let expected_result = LinkageResult {
            state,
            client_id: client.uid.clone(),
            server_id: server.uid.clone(),
        };

        // Call the function
        let result = compute_linkage(Rc::clone(&client), Rc::clone(&server), timeout);

        // Assert result matches expectation
        assert_eq!(result, expected_result);
    }
}
