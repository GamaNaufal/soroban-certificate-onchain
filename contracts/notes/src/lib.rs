#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, String, Symbol, Vec,
};

// ============================================================
// STRUKTUR DATA
// ============================================================

/// Satu sertifikat yang diterbitkan ke penerima
#[contracttype]
#[derive(Clone, Debug)]
pub struct Certificate {
    pub id: u64,
    /// Wallet address penerima sertifikat
    pub recipient: Address,
    /// Nama lengkap penerima
    pub recipient_name: String,
    /// Nama lembaga/institusi penerbit (misal: "Dicoding", "Universitas Indonesia")
    pub issuer_name: String,
    /// Wallet address penerbit — dipakai untuk auth saat revoke
    pub issuer_address: Address,
    /// Judul sertifikat (misal: "React Developer Intermediate")
    pub title: String,
    /// Kategori: "akademik", "vokasi", "bootcamp", dsb
    pub category: String,
    /// Tanggal terbit dalam unix timestamp (pakai env.ledger().timestamp())
    pub issued_at: u64,
    /// Apakah sertifikat masih berlaku
    pub is_valid: bool,
}

/// Ringkasan portofolio skill milik satu address
#[contracttype]
#[derive(Clone, Debug)]
pub struct Portfolio {
    pub owner: Address,
    /// Jumlah total sertifikat yang dimiliki
    pub total_certs: u32,
    /// Jumlah sertifikat yang masih valid
    pub valid_certs: u32,
}

// ============================================================
// STORAGE KEYS
// ============================================================

/// Menyimpan Vec<Certificate> — semua sertifikat yang pernah diterbitkan
const CERT_DATA: Symbol = symbol_short!("CERT_DATA");

/// Menyimpan Vec<Address> — daftar address yang terdaftar sebagai issuer resmi
const ISSUERS: Symbol = symbol_short!("ISSUERS");

/// Menyimpan Address pemilik/admin contract ini
const ADMIN: Symbol = symbol_short!("ADMIN");

// ============================================================
// CONTRACT
// ============================================================

#[contract]
pub struct CertificateContract;

#[contractimpl]
impl CertificateContract {

    // --------------------------------------------------------
    // INISIALISASI
    // --------------------------------------------------------

    /// Dipanggil sekali saat deploy. Mendaftarkan admin contract.
    pub fn initialize(env: Env, admin: Address) {
        // Pastikan belum pernah diinisialisasi
        if env.storage().instance().has(&ADMIN) {
            panic!("Contract sudah diinisialisasi");
        }
        admin.require_auth();
        env.storage().instance().set(&ADMIN, &admin);
    }

    // --------------------------------------------------------
    // MANAJEMEN ISSUER
    // --------------------------------------------------------

    /// Admin mendaftarkan lembaga sebagai issuer resmi
    pub fn add_issuer(env: Env, issuer: Address) -> String {
        // Hanya admin yang boleh
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        let mut issuers: Vec<Address> = env
            .storage()
            .instance()
            .get(&ISSUERS)
            .unwrap_or(Vec::new(&env));

        // Cegah duplikat
        for i in 0..issuers.len() {
            if issuers.get(i).unwrap() == issuer {
                return String::from_str(&env, "Issuer sudah terdaftar");
            }
        }

        issuers.push_back(issuer);
        env.storage().instance().set(&ISSUERS, &issuers);
        String::from_str(&env, "Issuer berhasil didaftarkan")
    }

    /// Admin mencabut status issuer
    pub fn remove_issuer(env: Env, issuer: Address) -> String {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();

        let mut issuers: Vec<Address> = env
            .storage()
            .instance()
            .get(&ISSUERS)
            .unwrap_or(Vec::new(&env));

        for i in 0..issuers.len() {
            if issuers.get(i).unwrap() == issuer {
                issuers.remove(i);
                env.storage().instance().set(&ISSUERS, &issuers);
                return String::from_str(&env, "Issuer berhasil dihapus");
            }
        }

        String::from_str(&env, "Issuer tidak ditemukan")
    }

    /// Cek apakah suatu address adalah issuer terdaftar
    pub fn is_registered_issuer(env: Env, issuer: Address) -> bool {
        let issuers: Vec<Address> = env
            .storage()
            .instance()
            .get(&ISSUERS)
            .unwrap_or(Vec::new(&env));

        for i in 0..issuers.len() {
            if issuers.get(i).unwrap() == issuer {
                return true;
            }
        }
        false
    }

    // --------------------------------------------------------
    // PENERBITAN SERTIFIKAT
    // --------------------------------------------------------

    /// Issuer menerbitkan sertifikat baru ke seorang penerima
    pub fn issue_certificate(
        env: Env,
        issuer: Address,
        issuer_name: String,
        recipient: Address,
        recipient_name: String,
        title: String,
        category: String,
    ) -> u64 {
        // Pastikan issuer sudah terdaftar
        if !Self::is_registered_issuer(env.clone(), issuer.clone()) {
            panic!("Hanya issuer terdaftar yang bisa menerbitkan sertifikat");
        }

        // Issuer harus menandatangani transaksi ini
        issuer.require_auth();

        let mut certs: Vec<Certificate> = env
            .storage()
            .instance()
            .get(&CERT_DATA)
            .unwrap_or(Vec::new(&env));

        let cert_id = env.prng().gen::<u64>();

        let cert = Certificate {
            id: cert_id,
            recipient: recipient.clone(),
            recipient_name,
            issuer_name,
            issuer_address: issuer,
            title,
            category,
            issued_at: env.ledger().timestamp(),
            is_valid: true,
        };

        certs.push_back(cert);
        env.storage().instance().set(&CERT_DATA, &certs);

        // Kembalikan ID sertifikat agar bisa disimpan issuer
        cert_id
    }

    // --------------------------------------------------------
    // REVOKE SERTIFIKAT
    // --------------------------------------------------------

    /// Issuer mencabut sertifikat yang pernah diterbitkan (misal: kecurangan)
    pub fn revoke_certificate(env: Env, issuer: Address, cert_id: u64) -> String {
        issuer.require_auth();

        let mut certs: Vec<Certificate> = env
            .storage()
            .instance()
            .get(&CERT_DATA)
            .unwrap_or(Vec::new(&env));

        for i in 0..certs.len() {
            let mut cert = certs.get(i).unwrap();
            if cert.id == cert_id {
                // Hanya issuer original yang bisa revoke
                if cert.issuer_address != issuer {
                    return String::from_str(&env, "Bukan penerbit sertifikat ini");
                }
                if !cert.is_valid {
                    return String::from_str(&env, "Sertifikat sudah dicabut sebelumnya");
                }
                cert.is_valid = false;
                certs.set(i, cert);
                env.storage().instance().set(&CERT_DATA, &certs);
                return String::from_str(&env, "Sertifikat berhasil dicabut");
            }
        }

        String::from_str(&env, "Sertifikat tidak ditemukan")
    }

    // --------------------------------------------------------
    // VERIFIKASI & QUERY
    // --------------------------------------------------------

    /// Verifikasi satu sertifikat berdasarkan ID — bisa dipanggil siapapun
    pub fn verify_certificate(env: Env, cert_id: u64) -> Option<Certificate> {
        let certs: Vec<Certificate> = env
            .storage()
            .instance()
            .get(&CERT_DATA)
            .unwrap_or(Vec::new(&env));

        for i in 0..certs.len() {
            let cert = certs.get(i).unwrap();
            if cert.id == cert_id {
                return Some(cert);
            }
        }
        None
    }

    /// Ambil semua sertifikat milik satu address (portofolio penerima)
    pub fn get_portfolio(env: Env, recipient: Address) -> Vec<Certificate> {
        let certs: Vec<Certificate> = env
            .storage()
            .instance()
            .get(&CERT_DATA)
            .unwrap_or(Vec::new(&env));

        let mut result = Vec::new(&env);
        for i in 0..certs.len() {
            let cert = certs.get(i).unwrap();
            if cert.recipient == recipient {
                result.push_back(cert);
            }
        }
        result
    }

    /// Ringkasan statistik portofolio satu address
    pub fn get_portfolio_summary(env: Env, recipient: Address) -> Portfolio {
        let portfolio = Self::get_portfolio(env.clone(), recipient.clone());

        let mut valid_count: u32 = 0;
        for i in 0..portfolio.len() {
            if portfolio.get(i).unwrap().is_valid {
                valid_count += 1;
            }
        }

        Portfolio {
            owner: recipient,
            total_certs: portfolio.len(),
            valid_certs: valid_count,
        }
    }

    /// Ambil semua sertifikat yang pernah diterbitkan oleh satu issuer
    pub fn get_issued_by(env: Env, issuer: Address) -> Vec<Certificate> {
        let certs: Vec<Certificate> = env
            .storage()
            .instance()
            .get(&CERT_DATA)
            .unwrap_or(Vec::new(&env));

        let mut result = Vec::new(&env);
        for i in 0..certs.len() {
            let cert = certs.get(i).unwrap();
            if cert.issuer_address == issuer {
                result.push_back(cert);
            }
        }
        result
    }

    /// Ambil semua sertifikat (untuk keperluan eksplorasi/admin)
    pub fn get_all_certificates(env: Env) -> Vec<Certificate> {
        env.storage()
            .instance()
            .get(&CERT_DATA)
            .unwrap_or(Vec::new(&env))
    }

    /// Cek apakah seorang penerima punya sertifikat valid untuk skill tertentu
    pub fn has_valid_certificate(
        env: Env,
        recipient: Address,
        title: String,
    ) -> bool {
        let portfolio = Self::get_portfolio(env.clone(), recipient);
        for i in 0..portfolio.len() {
            let cert = portfolio.get(i).unwrap();
            if cert.title == title && cert.is_valid {
                return true;
            }
        }
        false
    }
}

mod test;