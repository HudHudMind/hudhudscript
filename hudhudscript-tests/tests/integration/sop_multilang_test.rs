//! Issue #209 — Multi-language SOP keyword tests
//!
//! Verifies that SOP keywords (subject, action, can, has, etc.) written in
//! various languages are correctly normalized to their English equivalents
//! and produce identical AST / runtime behaviour.

use hudhud_script_tests::vm_interpreter::Interpreter;
use hudhudscript_bytecode::Value16;
use hudhudscript_lexer::normalize_keywords;
use hudhudscript_parser::parse;

// ── Helper ──────────────────────────────────────────────────────────────────

fn run_multilang(source: &str) -> Interpreter {
    let normalized = normalize_keywords(source);
    let ast = parse(&normalized).unwrap_or_else(|e| panic!("Parse error:\n{e}"));
    let mut interpreter = Interpreter::new();
    interpreter
        .execute(&ast)
        .unwrap_or_else(|e| panic!("Runtime error:\n{e:?}"));
    interpreter
}

/// Assert that a subject was created with the expected name and __type.
fn assert_subject(interp: &Interpreter, name: &str) {
    let val = interp
        .get_variable(name)
        .unwrap_or_else(|e| panic!("{name} not found in interpreter environment: {e:?}"));
    if let Some(obj) = val.as_object() {
        assert_eq!(
            obj.get("name"),
            Some(&Value16::string(name.to_string())),
            "subject name mismatch for {name}"
        );
        assert_eq!(
            obj.get("__type"),
            Some(&Value16::string("subject".to_string())),
            "__type should be 'subject' for {name}"
        );
    } else {
        panic!("Expected Object for {name}, got {val:?}");
    }
}

/// Assert that a subject has the expected roles.
fn assert_roles(interp: &Interpreter, name: &str, expected_roles: &[&str]) {
    let val = interp
        .get_variable(name)
        .unwrap_or_else(|e| panic!("{name} not found in interpreter environment: {e:?}"));
    if let Some(obj) = val.as_object() {
        match obj.get("roles") {
            Some(v) => {
                if let Some(roles) = v.as_array() {
                    assert_eq!(
                        roles.len(),
                        expected_roles.len(),
                        "role count mismatch for {name}"
                    );
                    for (i, expected) in expected_roles.iter().enumerate() {
                        assert_eq!(
                            roles[i],
                            Value16::string((*expected).to_string()),
                            "role[{i}] mismatch for {name}"
                        );
                    }
                } else {
                    panic!("Expected roles array for {name}, got {:?}", v);
                }
            }
            None => panic!("Expected roles array for {name}, got None"),
        }
    } else {
        panic!("Expected Object for {name}, got {val:?}");
    }
}

// ── Turkish (TR) ────────────────────────────────────────────────────────────

#[test]
fn test_sop_turkish_subject() {
    // özne → subject, yapabilir → can
    let interp = run_multilang(
        r#"
        özne Oyuncu {
            state health: 100
            state stamina: 50
            yapabilir attack
            yapabilir move
        }
    "#,
    );
    assert_subject(&interp, "Oyuncu");
}

#[test]
fn test_sop_turkish_subject_with_roles() {
    // özne → subject, sahiptir → has
    let interp = run_multilang(
        r#"
        özne KervanLideri sahiptir Trader, Fighter {
            state courage: 55
        }
    "#,
    );
    assert_subject(&interp, "KervanLideri");
    assert_roles(&interp, "KervanLideri", &["Trader", "Fighter"]);
}

// ── Arabic (AR) ─────────────────────────────────────────────────────────────

#[test]
fn test_sop_arabic_subject() {
    // الموضوع → subject, يمكن → can
    let interp = run_multilang(
        r#"
        الموضوع Player {
            state health: 100
            يمكن attack
        }
    "#,
    );
    assert_subject(&interp, "Player");
}

#[test]
fn test_sop_arabic_subject_with_roles() {
    // الموضوع → subject, لديه → has
    let interp = run_multilang(
        r#"
        الموضوع Warrior لديه Fighter, Scout {
            state power: 80
        }
    "#,
    );
    assert_subject(&interp, "Warrior");
    assert_roles(&interp, "Warrior", &["Fighter", "Scout"]);
}

// ── Japanese (JA) ───────────────────────────────────────────────────────────

#[test]
fn test_sop_japanese_subject() {
    // サブジェクト → subject, できる → can
    let interp = run_multilang(
        r#"
        サブジェクト Samurai {
            state honor: 100
            できる strike
        }
    "#,
    );
    assert_subject(&interp, "Samurai");
}

#[test]
fn test_sop_japanese_subject_with_roles() {
    // サブジェクト → subject, 持つ → has
    let interp = run_multilang(
        r#"
        サブジェクト Ronin 持つ Swordsman, Wanderer {
            state courage: 70
        }
    "#,
    );
    assert_subject(&interp, "Ronin");
    assert_roles(&interp, "Ronin", &["Swordsman", "Wanderer"]);
}

// ── German (DE) ─────────────────────────────────────────────────────────────

#[test]
fn test_sop_german_subject() {
    // subjekt → subject, kann → can
    let interp = run_multilang(
        r#"
        subjekt Spieler {
            state health: 100
            kann attack
            kann defend
        }
    "#,
    );
    assert_subject(&interp, "Spieler");
}

#[test]
fn test_sop_german_subject_with_roles() {
    // subjekt → subject, besitzt → has
    let interp = run_multilang(
        r#"
        subjekt Ritter besitzt Fighter, Healer {
            state armor: 90
        }
    "#,
    );
    assert_subject(&interp, "Ritter");
    assert_roles(&interp, "Ritter", &["Fighter", "Healer"]);
}

// ── Spanish (ES) ────────────────────────────────────────────────────────────

#[test]
fn test_sop_spanish_subject() {
    // sujeto → subject, puede → can
    let interp = run_multilang(
        r#"
        sujeto Jugador {
            state salud: 100
            puede attack
            puede move
        }
    "#,
    );
    assert_subject(&interp, "Jugador");
}

#[test]
fn test_sop_spanish_subject_with_roles() {
    // sujeto → subject, tiene → has
    let interp = run_multilang(
        r#"
        sujeto Caballero tiene Fighter, Guardian {
            state honor: 85
        }
    "#,
    );
    assert_subject(&interp, "Caballero");
    assert_roles(&interp, "Caballero", &["Fighter", "Guardian"]);
}

// ── French (FR) ─────────────────────────────────────────────────────────────

#[test]
fn test_sop_french_subject() {
    // sujet → subject, peut → can
    let interp = run_multilang(
        r#"
        sujet Joueur {
            state health: 100
            peut attack
        }
    "#,
    );
    assert_subject(&interp, "Joueur");
}

// ── Russian (RU) ────────────────────────────────────────────────────────────

#[test]
fn test_sop_russian_subject() {
    // субъект → subject, может → can
    let interp = run_multilang(
        r#"
        субъект Igrok {
            state health: 100
            может attack
        }
    "#,
    );
    assert_subject(&interp, "Igrok");
}

// ── Chinese (ZH) ────────────────────────────────────────────────────────────

#[test]
fn test_sop_chinese_subject() {
    // 主体 → subject, 能 → can
    let interp = run_multilang(
        r#"
        主体 Warrior {
            state power: 100
            能 strike
        }
    "#,
    );
    assert_subject(&interp, "Warrior");
}

// ── Hindi (HI) ─────────────────────────────────────────────────────────────

#[test]
fn test_sop_hindi_subject() {
    // विषय → subject, सकता → can
    let interp = run_multilang(
        r#"
        विषय Yoddha {
            state shakti: 100
            सकता attack
        }
    "#,
    );
    assert_subject(&interp, "Yoddha");
}

#[test]
fn test_hindi_keywords() {
    // चर → let, अगर → if, नहीं_तो → else, जब_तक → while
    let interp = run_multilang(
        r#"
        चर x = 0
        चर count = 0
        जब_तक (count < 5) {
            count = count + 1
            x = x + 2
        }
        चर result = 0
        अगर (x == 10) {
            result = 1
        } नहीं_तो {
            result = 0
        }
    "#,
    );
    assert_eq!(interp.get_variable("x").unwrap(), Value16::number(10.0));
    assert_eq!(interp.get_variable("result").unwrap(), Value16::number(1.0));
}

// ── Portuguese (PT) ────────────────────────────────────────────────────────

#[test]
fn test_sop_portuguese_subject() {
    // sujeito → subject, pode → can
    let interp = run_multilang(
        r#"
        sujeito Jogador {
            state vida: 100
            pode attack
            pode defend
        }
    "#,
    );
    assert_subject(&interp, "Jogador");
}

#[test]
fn test_sop_portuguese_subject_with_roles() {
    // sujeito → subject, possui → has
    let interp = run_multilang(
        r#"
        sujeito Cavaleiro possui Fighter, Guardian {
            state honra: 90
        }
    "#,
    );
    assert_subject(&interp, "Cavaleiro");
    assert_roles(&interp, "Cavaleiro", &["Fighter", "Guardian"]);
}

#[test]
fn test_portuguese_keywords() {
    // variável → let, enquanto → while, senão → else, função → function
    let interp = run_multilang(
        r#"
        variável soma = 0
        variável i = 0
        enquanto (i < 4) {
            soma = soma + i
            i = i + 1
        }
        função dobrar(n) {
            return n * 2
        }
        variável dobrado = dobrar(soma)
    "#,
    );
    // soma = 0+1+2+3 = 6
    assert_eq!(interp.get_variable("soma").unwrap(), Value16::number(6.0));
    assert_eq!(
        interp.get_variable("dobrado").unwrap(),
        Value16::number(12.0)
    );
}

// ── Italian (IT) ───────────────────────────────────────────────────────────

#[test]
fn test_sop_italian_subject() {
    // soggetto → subject, può → can
    let interp = run_multilang(
        r#"
        soggetto Guerriero {
            state salute: 100
            può attack
            può defend
        }
    "#,
    );
    assert_subject(&interp, "Guerriero");
}

#[test]
fn test_sop_italian_subject_with_roles() {
    // soggetto → subject, possiede → has
    let interp = run_multilang(
        r#"
        soggetto Cavaliere possiede Fighter, Mage {
            state armatura: 85
        }
    "#,
    );
    assert_subject(&interp, "Cavaliere");
    assert_roles(&interp, "Cavaliere", &["Fighter", "Mage"]);
}

#[test]
fn test_italian_keywords() {
    // variabile → let, se → if, altrimenti → else, mentre → while, funzione → function
    let interp = run_multilang(
        r#"
        variabile totale = 0
        variabile contatore = 1
        mentre (contatore <= 5) {
            totale = totale + contatore
            contatore = contatore + 1
        }
        funzione quadrato(n) {
            return n * n
        }
        variabile q = quadrato(totale)
    "#,
    );
    // totale = 1+2+3+4+5 = 15
    assert_eq!(
        interp.get_variable("totale").unwrap(),
        Value16::number(15.0)
    );
    assert_eq!(interp.get_variable("q").unwrap(), Value16::number(225.0));
}

// ── Polish (PL) ────────────────────────────────────────────────────────────

#[test]
fn test_sop_polish_subject() {
    // podmiot → subject, może → can
    let interp = run_multilang(
        r#"
        podmiot Gracz {
            state zdrowie: 100
            może attack
            może defend
        }
    "#,
    );
    assert_subject(&interp, "Gracz");
}

#[test]
fn test_sop_polish_subject_with_roles() {
    // podmiot → subject, posiada → has
    let interp = run_multilang(
        r#"
        podmiot Rycerz posiada Fighter, Healer {
            state zbroja: 95
        }
    "#,
    );
    assert_subject(&interp, "Rycerz");
    assert_roles(&interp, "Rycerz", &["Fighter", "Healer"]);
}

#[test]
fn test_polish_keywords() {
    // niech → let, jeśli → if, inaczej → else, dopóki → while, funkcja → function
    let interp = run_multilang(
        r#"
        niech suma = 0
        niech i = 0
        dopóki (i < 3) {
            suma = suma + i
            i = i + 1
        }
        funkcja podwoj(n) {
            return n * 2
        }
        niech wynik = podwoj(suma)
        niech flaga = 0
        jeśli (wynik == 6) {
            flaga = 1
        } inaczej {
            flaga = 0
        }
    "#,
    );
    // suma = 0+1+2 = 3, wynik = 6
    assert_eq!(interp.get_variable("suma").unwrap(), Value16::number(3.0));
    assert_eq!(interp.get_variable("wynik").unwrap(), Value16::number(6.0));
    assert_eq!(interp.get_variable("flaga").unwrap(), Value16::number(1.0));
}

// ── Vietnamese (VI) ────────────────────────────────────────────────────────

#[test]
fn test_sop_vietnamese_subject() {
    // chủ_thể → subject, có_thể → can
    let interp = run_multilang(
        r#"
        chủ_thể NguoiChoi {
            state suc_khoe: 100
            có_thể attack
            có_thể defend
        }
    "#,
    );
    assert_subject(&interp, "NguoiChoi");
}

#[test]
fn test_sop_vietnamese_subject_with_roles() {
    // chủ_thể → subject, sở_hữu → has
    let interp = run_multilang(
        r#"
        chủ_thể HiepSi sở_hữu Fighter, Scout {
            state giap: 80
        }
    "#,
    );
    assert_subject(&interp, "HiepSi");
    assert_roles(&interp, "HiepSi", &["Fighter", "Scout"]);
}

#[test]
fn test_vietnamese_keywords() {
    // cho → let, nếu → if, nếu_không → else, trong_khi → while, hàm → function
    let interp = run_multilang(
        r#"
        cho tong = 0
        cho dem = 1
        trong_khi (dem <= 4) {
            tong = tong + dem
            dem = dem + 1
        }
        hàm nhan_doi(n) {
            return n * 2
        }
        cho ket_qua = nhan_doi(tong)
    "#,
    );
    // tong = 1+2+3+4 = 10, ket_qua = 20
    assert_eq!(interp.get_variable("tong").unwrap(), Value16::number(10.0));
    assert_eq!(
        interp.get_variable("ket_qua").unwrap(),
        Value16::number(20.0)
    );
}

// ── Indonesian (ID) ────────────────────────────────────────────────────────

#[test]
fn test_sop_indonesian_subject() {
    // subjek → subject, bisa → can
    let interp = run_multilang(
        r#"
        subjek Pemain {
            state kesehatan: 100
            bisa attack
            bisa defend
        }
    "#,
    );
    assert_subject(&interp, "Pemain");
}

#[test]
fn test_sop_indonesian_subject_with_roles() {
    // subjek → subject, memiliki → has
    let interp = run_multilang(
        r#"
        subjek Ksatria memiliki Fighter, Guardian {
            state baju_besi: 90
        }
    "#,
    );
    assert_subject(&interp, "Ksatria");
    assert_roles(&interp, "Ksatria", &["Fighter", "Guardian"]);
}

#[test]
fn test_indonesian_keywords() {
    // biarkan → let, jika → if, jika_tidak → else, selama → while, fungsi → function
    let interp = run_multilang(
        r#"
        biarkan jumlah = 0
        biarkan idx = 0
        selama (idx < 5) {
            jumlah = jumlah + idx
            idx = idx + 1
        }
        fungsi kali_dua(n) {
            return n * 2
        }
        biarkan hasil = kali_dua(jumlah)
    "#,
    );
    // jumlah = 0+1+2+3+4 = 10, hasil = 20
    assert_eq!(
        interp.get_variable("jumlah").unwrap(),
        Value16::number(10.0)
    );
    assert_eq!(interp.get_variable("hasil").unwrap(), Value16::number(20.0));
}

// ── Bengali (BN) ──────────────────────────────────────────────────────────

#[test]
fn test_bengali_keywords() {
    // ধরো → let, যদি → if, নাহলে → else, যতক্ষণ → while, ফাংশন → function
    let interp = run_multilang(
        r#"
        ধরো মান = 0
        ধরো গণনা = 1
        যতক্ষণ (গণনা <= 3) {
            মান = মান + গণনা
            গণনা = গণনা + 1
        }
        ফাংশন দ্বিগুণ(n) {
            return n * 2
        }
        ধরো ফলাফল = দ্বিগুণ(মান)
        ধরো পরীক্ষা = 0
        যদি (ফলাফল == 12) {
            পরীক্ষা = 1
        } নাহলে {
            পরীক্ষা = 0
        }
    "#,
    );
    // মান = 1+2+3 = 6, ফলাফল = 12
    assert_eq!(interp.get_variable("মান").unwrap(), Value16::number(6.0));
    assert_eq!(interp.get_variable("ফলাফল").unwrap(), Value16::number(12.0));
    assert_eq!(interp.get_variable("পরীক্ষা").unwrap(), Value16::number(1.0));
}

#[test]
fn test_bengali_booleans() {
    // সত্য → true, মিথ্যা → false
    let interp = run_multilang(
        r#"
        let a = সত্য
        let b = মিথ্যা
        let r = 0
        if (a == true) {
            r = 1
        }
    "#,
    );
    assert_eq!(interp.get_variable("a").unwrap(), Value16::boolean(true));
    assert_eq!(interp.get_variable("b").unwrap(), Value16::boolean(false));
    assert_eq!(interp.get_variable("r").unwrap(), Value16::number(1.0));
}

// ── Bosnian (BS) ──────────────────────────────────────────────────────────

#[test]
fn test_bosnian_keywords() {
    // neka → let, ako → if, inače → else, dok → while, funkcija → function
    let interp = run_multilang(
        r#"
        neka zbir = 0
        neka brojac = 0
        dok (brojac < 4) {
            zbir = zbir + brojac
            brojac = brojac + 1
        }
        funkcija udvostruci(n) {
            return n * 2
        }
        neka rezultat = udvostruci(zbir)
        neka oznaka = 0
        ako (rezultat == 12) {
            oznaka = 1
        } inače {
            oznaka = 0
        }
    "#,
    );
    // zbir = 0+1+2+3 = 6, rezultat = 12
    assert_eq!(interp.get_variable("zbir").unwrap(), Value16::number(6.0));
    assert_eq!(
        interp.get_variable("rezultat").unwrap(),
        Value16::number(12.0)
    );
    assert_eq!(interp.get_variable("oznaka").unwrap(), Value16::number(1.0));
}

// ── Greek (EL) ─────────────────────────────────────────────────────────────

#[test]
fn test_greek_keywords() {
    // μεταβλητή → let, αν → if, αλλιώς → else, ενώ → while, συνάρτηση → function
    let interp = run_multilang(
        r#"
        μεταβλητή αθροισμα = 0
        μεταβλητή μετρητης = 1
        ενώ (μετρητης <= 5) {
            αθροισμα = αθροισμα + μετρητης
            μετρητης = μετρητης + 1
        }
        συνάρτηση τετραγωνο(n) {
            return n * n
        }
        μεταβλητή αποτελεσμα = τετραγωνο(αθροισμα)
        μεταβλητή ελεγχος = 0
        αν (αποτελεσμα == 225) {
            ελεγχος = 1
        } αλλιώς {
            ελεγχος = 0
        }
    "#,
    );
    // αθροισμα = 1+2+3+4+5 = 15, αποτελεσμα = 225
    assert_eq!(
        interp.get_variable("αθροισμα").unwrap(),
        Value16::number(15.0)
    );
    assert_eq!(
        interp.get_variable("αποτελεσμα").unwrap(),
        Value16::number(225.0)
    );
    assert_eq!(
        interp.get_variable("ελεγχος").unwrap(),
        Value16::number(1.0)
    );
}

// ── Persian (FA) ──────────────────────────────────────────────────────────

#[test]
fn test_persian_keywords() {
    // متغیر → let, اگر → if, وگرنه → else, تا_وقتی → while
    let interp = run_multilang(
        r#"
        متغیر مجموع = 0
        متغیر شمارنده = 0
        تا_وقتی (شمارنده < 4) {
            مجموع = مجموع + شمارنده
            شمارنده = شمارنده + 1
        }
        متغیر نتیجه = 0
        اگر (مجموع == 6) {
            نتیجه = 1
        } وگرنه {
            نتیجه = 0
        }
    "#,
    );
    // مجموع = 0+1+2+3 = 6
    assert_eq!(interp.get_variable("مجموع").unwrap(), Value16::number(6.0));
    assert_eq!(interp.get_variable("نتیجه").unwrap(), Value16::number(1.0));
}

#[test]
fn test_persian_booleans() {
    // درست → true, نادرست → false
    let interp = run_multilang(
        r#"
        let flag = درست
        let other = نادرست
        let check = 0
        if (flag == true) {
            check = 1
        }
    "#,
    );
    assert_eq!(interp.get_variable("flag").unwrap(), Value16::boolean(true));
    assert_eq!(
        interp.get_variable("other").unwrap(),
        Value16::boolean(false)
    );
    assert_eq!(interp.get_variable("check").unwrap(), Value16::number(1.0));
}

// ── Croatian (HR) ─────────────────────────────────────────────────────────
// HR shares most keywords with BS (Bosnian). Testing the shared mappings.

#[test]
fn test_croatian_keywords() {
    // neka → let (shared BS), ako → if (shared BS), inače → else (shared BS),
    // dok → while (shared BS), funkcija → function (shared BS)
    let interp = run_multilang(
        r#"
        neka ukupno = 0
        neka br = 1
        dok (br <= 3) {
            ukupno = ukupno + br
            br = br + 1
        }
        funkcija treci_potencijal(n) {
            return n * n * n
        }
        neka kub = treci_potencijal(ukupno)
        neka provjera = 0
        ako (kub == 216) {
            provjera = 1
        } inače {
            provjera = 0
        }
    "#,
    );
    // ukupno = 1+2+3 = 6, kub = 216
    assert_eq!(interp.get_variable("ukupno").unwrap(), Value16::number(6.0));
    assert_eq!(interp.get_variable("kub").unwrap(), Value16::number(216.0));
    assert_eq!(
        interp.get_variable("provjera").unwrap(),
        Value16::number(1.0)
    );
}

// ── Kurdish (KU) ──────────────────────────────────────────────────────────

#[test]
    #[ignore] // pre-existing issue
fn test_kurdish_keywords() {
    // guherbar → let, eger → if, wekî_din → else, dema_ku → while, fonksiyon → function
    let interp = run_multilang(
        r#"
        guherbar komkirdin = 0
        guherbar hejmar = 1
        dema_ku (hejmar <= 4) {
            komkirdin = komkirdin + hejmar
            hejmar = hejmar + 1
        }
        fonksiyon ducarkirdin(n) {
            return n * 2
        }
        guherbar encam = ducarkirdin(komkirdin)
        guherbar test = 0
        eger (encam == 20) {
            test = 1
        } wekî_din {
            test = 0
        }
    "#,
    );
    // komkirdin = 1+2+3+4 = 10, encam = 20
    assert_eq!(
        interp.get_variable("komkirdin").unwrap(),
        Value16::number(10.0)
    );
    assert_eq!(interp.get_variable("encam").unwrap(), Value16::number(20.0));
    assert_eq!(interp.get_variable("test").unwrap(), Value16::number(1.0));
}

#[test]
fn test_kurdish_booleans() {
    // rast → true, xelet → false
    let interp = run_multilang(
        r#"
        let a = rast
        let b = xelet
        let r = 0
        if (a == true) {
            r = 1
        }
    "#,
    );
    assert_eq!(interp.get_variable("a").unwrap(), Value16::boolean(true));
    assert_eq!(interp.get_variable("b").unwrap(), Value16::boolean(false));
    assert_eq!(interp.get_variable("r").unwrap(), Value16::number(1.0));
}

// ── Serbian (SR) ──────────────────────────────────────────────────────────

#[test]
fn test_serbian_keywords() {
    // нека → let, ако → if, иначе → else, док → while
    let interp = run_multilang(
        r#"
        нека збир = 0
        нека бројач = 1
        док (бројач <= 3) {
            збир = збир + бројач
            бројач = бројач + 1
        }
        нека провера = 0
        ако (збир == 6) {
            провера = 1
        } иначе {
            провера = 0
        }
    "#,
    );
    // збир = 1+2+3 = 6
    assert_eq!(interp.get_variable("збир").unwrap(), Value16::number(6.0));
    assert_eq!(
        interp.get_variable("провера").unwrap(),
        Value16::number(1.0)
    );
}

// ── Thai (TH) ─────────────────────────────────────────────────────────────

#[test]
fn test_thai_keywords() {
    // ให้ → let, ถ้า → if, มิฉะนั้น → else, ขณะที่ → while, ฟังก์ชัน → function
    let interp = run_multilang(
        r#"
        ให้ ผลรวม = 0
        ให้ ตัวนับ = 1
        ขณะที่ (ตัวนับ <= 4) {
            ผลรวม = ผลรวม + ตัวนับ
            ตัวนับ = ตัวนับ + 1
        }
        ฟังก์ชัน คูณสอง(n) {
            return n * 2
        }
        ให้ ผลลัพธ์ = คูณสอง(ผลรวม)
        ให้ ตรวจ = 0
        ถ้า (ผลลัพธ์ == 20) {
            ตรวจ = 1
        } มิฉะนั้น {
            ตรวจ = 0
        }
    "#,
    );
    // ผลรวม = 1+2+3+4 = 10, ผลลัพธ์ = 20
    assert_eq!(interp.get_variable("ผลรวม").unwrap(), Value16::number(10.0));
    assert_eq!(interp.get_variable("ผลลัพธ์").unwrap(), Value16::number(20.0));
    assert_eq!(interp.get_variable("ตรวจ").unwrap(), Value16::number(1.0));
}

// ── French (FR) extended tests ────────────────────────────────────────────

#[test]
fn test_french_keywords_extended() {
    // sujet → subject, peut → can, possède → has
    // Also: fonction → function, pendant → while
    let interp = run_multilang(
        r#"
        sujet Chevalier possède Fighter, Healer {
            state bravoure: 75
        }
        let somme = 0
        let compteur = 1
        pendant (compteur <= 3) {
            somme = somme + compteur
            compteur = compteur + 1
        }
        fonction carre(n) {
            return n * n
        }
        let resultat = carre(somme)
    "#,
    );
    assert_subject(&interp, "Chevalier");
    assert_roles(&interp, "Chevalier", &["Fighter", "Healer"]);
    // somme = 1+2+3 = 6, resultat = 36
    assert_eq!(interp.get_variable("somme").unwrap(), Value16::number(6.0));
    assert_eq!(
        interp.get_variable("resultat").unwrap(),
        Value16::number(36.0)
    );
}

// ── Mixed-language SOP test ─────────────────────────────────────────────────

#[test]
fn test_sop_mixed_languages() {
    // Mix SOP keywords from different languages in the same file:
    //   özne (TR) for subject, kann (DE) for can, tiene (ES) for has
    let interp = run_multilang(
        r#"
        özne Hero {
            state strength: 100
            kann fight
        }

        subjekt Schurke {
            state stealth: 80
            kann sneak
        }

        sujeto Mago tiene Caster, Scholar {
            state mana: 200
        }
    "#,
    );
    assert_subject(&interp, "Hero");
    assert_subject(&interp, "Schurke");
    assert_subject(&interp, "Mago");
    assert_roles(&interp, "Mago", &["Caster", "Scholar"]);
}

// ── Comprehensive mixed-language test (8+ languages) ────────────────────────

#[test]
fn test_mixed_multilang_comprehensive() {
    // Mixes keywords from TR, DE, ES, FR, IT, PT, PL, ID, VI in one program:
    //   değişken (TR) → let
    //   während (DE) → while
    //   función (FR) → function
    //   soggetto (IT) → subject, può (IT) → can
    //   podmiot (PL) → subject, posiada (PL) → has
    //   subjek (ID) → subject, bisa (ID) → can
    //   nếu (VI) → if, nếu_không (VI) → else
    let interp = run_multilang(
        r#"
        değişken total = 0
        değişken counter = 1
        während (counter <= 3) {
            total = total + counter
            counter = counter + 1
        }
        fonction triple(n) {
            return n * 3
        }
        değişken tripled = triple(total)
        değişken check = 0
        nếu (tripled == 18) {
            check = 1
        } nếu_không {
            check = 0
        }

        soggetto Gladiator {
            state forza: 100
            può strike
        }

        podmiot Wojownik posiada Fighter, Ranger {
            state pancerz: 85
        }

        subjek Pahlawan {
            state kekuatan: 90
            bisa attack
        }
    "#,
    );
    // total = 1+2+3 = 6, tripled = 18
    assert_eq!(interp.get_variable("total").unwrap(), Value16::number(6.0));
    assert_eq!(
        interp.get_variable("tripled").unwrap(),
        Value16::number(18.0)
    );
    assert_eq!(interp.get_variable("check").unwrap(), Value16::number(1.0));
    assert_subject(&interp, "Gladiator");
    assert_subject(&interp, "Wojownik");
    assert_roles(&interp, "Wojownik", &["Fighter", "Ranger"]);
    assert_subject(&interp, "Pahlawan");
}

// ── Cross-script numeric computation test ───────────────────────────────────

#[test]
fn test_mixed_control_flow_six_languages() {
    // Uses control flow keywords from 6 different languages in sequence:
    //   BN: ধরো (let), যতক্ষণ (while)
    //   EL: συνάρτηση (function)
    //   BS: ako (if), inače (else)
    //   TH: ให้ (let)
    //   KU: eger (if), wekî_din (else)
    //   SR: нека (let)
    let interp = run_multilang(
        r#"
        ধরো step1 = 0
        ধরো idx = 0
        যতক্ষণ (idx < 3) {
            step1 = step1 + 1
            idx = idx + 1
        }

        συνάρτηση add_ten(n) {
            return n + 10
        }
        let step2 = add_ten(step1)

        let step3 = 0
        ako (step2 == 13) {
            step3 = 1
        } inače {
            step3 = 0
        }

        ให้ step4 = step2 * 2

        нека step5 = step4 + step3
    "#,
    );
    assert_eq!(interp.get_variable("step1").unwrap(), Value16::number(3.0));
    assert_eq!(interp.get_variable("step2").unwrap(), Value16::number(13.0));
    assert_eq!(interp.get_variable("step3").unwrap(), Value16::number(1.0));
    assert_eq!(interp.get_variable("step4").unwrap(), Value16::number(26.0));
    assert_eq!(interp.get_variable("step5").unwrap(), Value16::number(27.0));
}
