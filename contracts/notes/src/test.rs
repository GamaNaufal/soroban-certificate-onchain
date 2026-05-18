#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, Env, String};
    use crate::{CertificateContract, CertificateContractClient};

    // Helper: setup environment + deploy contract + inisialisasi admin
    fn setup() -> (Env, Address, CertificateContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths(); // Mock semua auth agar test tidak perlu wallet asli

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, CertificateContract);
        let client = CertificateContractClient::new(&env, &contract_id);

        client.initialize(&admin);
        (env, admin, client)
    }

    // --------------------------------------------------------
    // TEST 1: Inisialisasi contract
    // --------------------------------------------------------
    #[test]
    fn test_initialize() {
        let (_env, _admin, _client) = setup();
        // Jika tidak panic, inisialisasi berhasil
    }

    #[test]
    #[should_panic(expected = "Contract sudah diinisialisasi")]
    fn test_initialize_twice_should_panic() {
        let (env, admin, client) = setup();
        let admin2 = Address::generate(&env);
        client.initialize(&admin2); // Harus panic
        let _ = admin;
    }

    // --------------------------------------------------------
    // TEST 2: Manajemen issuer
    // --------------------------------------------------------
    #[test]
    fn test_add_and_check_issuer() {
        let (env, admin, client) = setup();
        let issuer = Address::generate(&env);

        let result = client.add_issuer(&admin, &issuer);
        assert_eq!(result, String::from_str(&env, "Issuer berhasil didaftarkan"));
        assert!(client.is_registered_issuer(&issuer));
    }

    #[test]
    fn test_add_duplicate_issuer() {
        let (env, admin, client) = setup();
        let issuer = Address::generate(&env);

        client.add_issuer(&admin, &issuer);
        let result = client.add_issuer(&admin, &issuer);
        assert_eq!(result, String::from_str(&env, "Issuer sudah terdaftar"));
    }

    #[test]
    fn test_remove_issuer() {
        let (env, admin, client) = setup();
        let issuer = Address::generate(&env);

        client.add_issuer(&admin, &issuer);
        assert!(client.is_registered_issuer(&issuer));

        let result = client.remove_issuer(&admin, &issuer);
        assert_eq!(result, String::from_str(&env, "Issuer berhasil dihapus"));
        assert!(!client.is_registered_issuer(&issuer));
    }

    // --------------------------------------------------------
    // TEST 3: Penerbitan sertifikat
    // --------------------------------------------------------
    #[test]
    fn test_issue_certificate() {
        let (env, admin, client) = setup();
        let issuer = Address::generate(&env);
        let recipient = Address::generate(&env);

        client.add_issuer(&admin, &issuer);

        let cert_id = client.issue_certificate(
            &issuer,
            &String::from_str(&env, "Dicoding Indonesia"),
            &recipient,
            &String::from_str(&env, "Budi Santoso"),
            &String::from_str(&env, "React Developer Intermediate"),
            &String::from_str(&env, "bootcamp"),
        );

        // Verifikasi sertifikat yang baru dibuat
        let cert = client.verify_certificate(&cert_id).unwrap();
        assert_eq!(cert.id, cert_id);
        assert_eq!(cert.recipient, recipient);
        assert_eq!(cert.issuer_address, issuer);
        assert!(cert.is_valid);
    }

    #[test]
    #[should_panic(expected = "Hanya issuer terdaftar yang bisa menerbitkan sertifikat")]
    fn test_issue_by_unregistered_issuer_should_panic() {
        let (env, _admin, client) = setup();
        let fake_issuer = Address::generate(&env);
        let recipient = Address::generate(&env);

        // Harus panic karena fake_issuer tidak terdaftar
        client.issue_certificate(
            &fake_issuer,
            &String::from_str(&env, "Lembaga Palsu"),
            &recipient,
            &String::from_str(&env, "Penerima"),
            &String::from_str(&env, "Sertifikat Palsu"),
            &String::from_str(&env, "lainnya"),
        );
    }

    // --------------------------------------------------------
    // TEST 4: Revoke sertifikat
    // --------------------------------------------------------
    #[test]
    fn test_revoke_certificate() {
        let (env, admin, client) = setup();
        let issuer = Address::generate(&env);
        let recipient = Address::generate(&env);

        client.add_issuer(&admin, &issuer);
        let cert_id = client.issue_certificate(
            &issuer,
            &String::from_str(&env, "Dicoding"),
            &recipient,
            &String::from_str(&env, "Andi"),
            &String::from_str(&env, "Flutter Pemula"),
            &String::from_str(&env, "bootcamp"),
        );

        let result = client.revoke_certificate(&issuer, &cert_id);
        assert_eq!(result, String::from_str(&env, "Sertifikat berhasil dicabut"));

        // Sertifikat masih ada tapi is_valid = false
        let cert = client.verify_certificate(&cert_id).unwrap();
        assert!(!cert.is_valid);
    }

    #[test]
    fn test_revoke_by_wrong_issuer() {
        let (env, admin, client) = setup();
        let issuer_a = Address::generate(&env);
        let issuer_b = Address::generate(&env);
        let recipient = Address::generate(&env);

        client.add_issuer(&admin, &issuer_a);
        client.add_issuer(&admin, &issuer_b);

        let cert_id = client.issue_certificate(
            &issuer_a,
            &String::from_str(&env, "Lembaga A"),
            &recipient,
            &String::from_str(&env, "Siti"),
            &String::from_str(&env, "Python Dasar"),
            &String::from_str(&env, "vokasi"),
        );

        // Issuer B mencoba revoke sertifikat milik issuer A — harus gagal
        let result = client.revoke_certificate(&issuer_b, &cert_id);
        assert_eq!(result, String::from_str(&env, "Bukan penerbit sertifikat ini"));
    }

    // --------------------------------------------------------
    // TEST 5: Portofolio penerima
    // --------------------------------------------------------
    #[test]
    fn test_get_portfolio() {
        let (env, admin, client) = setup();
        let issuer = Address::generate(&env);
        let recipient = Address::generate(&env);

        client.add_issuer(&admin, &issuer);

        // Terbitkan 3 sertifikat ke recipient yang sama
        client.issue_certificate(&issuer, &String::from_str(&env, "Dicoding"),
            &recipient, &String::from_str(&env, "Rina"),
            &String::from_str(&env, "React Dasar"), &String::from_str(&env, "bootcamp"));

        client.issue_certificate(&issuer, &String::from_str(&env, "Dicoding"),
            &recipient, &String::from_str(&env, "Rina"),
            &String::from_str(&env, "Node.js Dasar"), &String::from_str(&env, "bootcamp"));

        client.issue_certificate(&issuer, &String::from_str(&env, "Dicoding"),
            &recipient, &String::from_str(&env, "Rina"),
            &String::from_str(&env, "Docker Fundamental"), &String::from_str(&env, "devops"));

        let portfolio = client.get_portfolio(&recipient);
        assert_eq!(portfolio.len(), 3);

        let summary = client.get_portfolio_summary(&recipient);
        assert_eq!(summary.total_certs, 3);
        assert_eq!(summary.valid_certs, 3);
    }

    // --------------------------------------------------------
    // TEST 6: has_valid_certificate — use case employer
    // --------------------------------------------------------
    #[test]
    fn test_has_valid_certificate() {
        let (env, admin, client) = setup();
        let issuer = Address::generate(&env);
        let kandidat = Address::generate(&env);

        client.add_issuer(&admin, &issuer);
        let cert_id = client.issue_certificate(
            &issuer,
            &String::from_str(&env, "Hacktiv8"),
            &kandidat,
            &String::from_str(&env, "Doni Prasetyo"),
            &String::from_str(&env, "Full Stack JavaScript"),
            &String::from_str(&env, "bootcamp"),
        );

        // Employer cek: apakah kandidat punya sertifikat Full Stack JavaScript yang valid?
        assert!(client.has_valid_certificate(
            &kandidat,
            &String::from_str(&env, "Full Stack JavaScript")
        ));

        // Revoke lalu cek lagi
        client.revoke_certificate(&issuer, &cert_id);
        assert!(!client.has_valid_certificate(
            &kandidat,
            &String::from_str(&env, "Full Stack JavaScript")
        ));
    }
}