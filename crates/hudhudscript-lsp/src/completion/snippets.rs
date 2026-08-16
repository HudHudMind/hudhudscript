//! Snippet (template) definitions for HudHudScript completion.
//!
//! Each snippet uses LSP tab-stop syntax like `${1:condition}` and `$0`.

/// A code-completion snippet.
#[derive(Debug, Clone, Copy)]
pub struct Snippet {
    pub label: &'static str,
    pub detail: &'static str,
    pub insert_text: &'static str,
}

// ---------------------------------------------------------------------------
// Declarations
// ---------------------------------------------------------------------------

pub const DECLARATION_SNIPPETS_EN: &[Snippet] = &[
    Snippet {
        label: "agent",
        detail: "Agent declaration",
        insert_text: "agent ${1:Name} {\n\tmodel \"${2:gpt-4o}\"\n\t$0\n}",
    },
    Snippet {
        label: "provider",
        detail: "Provider declaration",
        insert_text: "provider ${1:Name} {\n\tbase_url: \"${2:https://api.example.com}\"\n\t$0\n}",
    },
    Snippet {
        label: "subject",
        detail: "Subject declaration",
        insert_text: "subject ${1:Name} {\n\tstate ${2:field}: ${3:value}\n\t$0\n}",
    },
    Snippet {
        label: "entity",
        detail: "Entity declaration",
        insert_text: "entity ${1:Name} {\n\tdata ${2:field}: ${3:Type}\n\t$0\n}",
    },
    Snippet {
        label: "class",
        detail: "Class declaration",
        insert_text: "class ${1:Name} {\n\tconstructor(${2:params}) {\n\t\t$0\n\t}\n}",
    },
    Snippet {
        label: "constitution",
        detail: "Governance constitution declaration",
        insert_text: "constitution ${1:Name} {\n\tlaw ${2:ruleName} {\n\t\t$0\n\t}\n}",
    },
    Snippet {
        label: "protocol",
        detail: "Execution protocol declaration",
        insert_text: "protocol ${1:Name} {\n\trole ${2:RoleName} {\n\t\t$0\n\t}\n}",
    },
    Snippet {
        label: "loop",
        detail: "Loop engineering block declaration",
        insert_text: "loop ${1:name} {\n\tstep ${2:stepName} {\n\t\t$0\n\t}\n}",
    },
    Snippet {
        label: "store",
        detail: "RAG vector store declaration",
        insert_text: "store ${1:Name} {\n\tembed \"${2:model}\"\n\t$0\n}",
    },
];

pub const DECLARATION_SNIPPETS_TR: &[Snippet] = &[
    Snippet {
        label: "ajan",
        detail: "Ajan tanımı",
        insert_text: "ajan ${1:İsim} {\n\tmodel \"${2:gpt-4o}\"\n\t$0\n}",
    },
    Snippet {
        label: "sağlayıcı",
        detail: "Sağlayıcı tanımı",
        insert_text: "sağlayıcı ${1:İsim} {\n\tbase_url: \"${2:https://api.ornek.com}\"\n\t$0\n}",
    },
    Snippet {
        label: "varlık",
        detail: "Varlık tanımı",
        insert_text: "varlık ${1:İsim} {\n\tveri ${2:alan}: ${3:Tür}\n\t$0\n}",
    },
    Snippet {
        label: "sınıf",
        detail: "Sınıf tanımı",
        insert_text: "sınıf ${1:İsim} {\n\tkurucu(${2:parametreler}) {\n\t\t$0\n\t}\n}",
    },
    Snippet {
        label: "anayasa",
        detail: "Yönetim anayasası tanımı",
        insert_text: "anayasa ${1:İsim} {\n\tyasa ${2:kuralAdı} {\n\t\t$0\n\t}\n}",
    },
    Snippet {
        label: "protokol",
        detail: "Yürütme protokolü tanımı",
        insert_text: "protokol ${1:İsim} {\n\trol ${2:RolAdı} {\n\t\t$0\n\t}\n}",
    },
    Snippet {
        label: "döngü-blok",
        detail: "Döngü mühendisliği bloğu",
        insert_text: "döngü ${1:isim} {\n\tadım ${2:adımAdı} {\n\t\t$0\n\t}\n}",
    },
    Snippet {
        label: "kayıt-yeri",
        detail: "RAG vektör kayıt yeri tanımı",
        insert_text: "kayıt_yeri ${1:İsim} {\n\tembed \"${2:model}\"\n\t$0\n}",
    },
];

pub const DECLARATION_SNIPPETS_AR: &[Snippet] = &[
    Snippet {
        label: "وكيل",
        detail: "تعريف وكيل",
        insert_text: "وكيل ${1:الاسم} {\n\tنموذج \"${2:gpt-4o}\"\n\t$0\n}",
    },
    Snippet {
        label: "مزود",
        detail: "تعريف مزود",
        insert_text: "مزود ${1:الاسم} {\n\tbase_url: \"${2:https://api.example.com}\"\n\t$0\n}",
    },
    Snippet {
        label: "الموضوع",
        detail: "تعريف موضوع",
        insert_text: "الموضوع ${1:الاسم} {\n\tحالة ${2:حقل}: ${3:قيمة}\n\t$0\n}",
    },
    Snippet {
        label: "كيان",
        detail: "تعريف كيان",
        insert_text: "كيان ${1:الاسم} {\n\tبيانات ${2:حقل}: ${3:نوع}\n\t$0\n}",
    },
    Snippet {
        label: "صنف",
        detail: "تعريف صنف",
        insert_text: "صنف ${1:الاسم} {\n\tconstructor(${2:المعاملات}) {\n\t\t$0\n\t}\n}",
    },
    Snippet {
        label: "دستور",
        detail: "تعريف دستور حكم",
        insert_text: "دستور ${1:الاسم} {\n\tقانون ${2:اسم_القاعدة} {\n\t\t$0\n\t}\n}",
    },
    Snippet {
        label: "بروتوكول",
        detail: "تعريف بروتوكول تنفيذ",
        insert_text: "بروتوكول ${1:الاسم} {\n\tدور ${2:اسم_الدور} {\n\t\t$0\n\t}\n}",
    },
    Snippet {
        label: "حلقة",
        detail: "تعريف كتلة هندسة الحلقات",
        insert_text: "حلقة ${1:الاسم} {\n\tخطوة ${2:اسم_الخطوة} {\n\t\t$0\n\t}\n}",
    },
    Snippet {
        label: "مخزن",
        detail: "تعريف مخزن متجه RAG",
        insert_text: "مخزن ${1:الاسم} {\n\tembed \"${2:model}\"\n\t$0\n}",
    },
];

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

pub const STATEMENT_SNIPPETS_EN: &[Snippet] = &[
    Snippet {
        label: "if",
        detail: "If statement",
        insert_text: "if (${1:condition}) {\n\t$0\n}",
    },
    Snippet {
        label: "if-else",
        detail: "If-else statement",
        insert_text: "if (${1:condition}) {\n\t$2\n} else {\n\t$0\n}",
    },
    Snippet {
        label: "else-if",
        detail: "Else-if ladder",
        insert_text: "else if (${1:condition}) {\n\t$0\n}",
    },
    Snippet {
        label: "while",
        detail: "While loop",
        insert_text: "while (${1:condition}) {\n\t$0\n}",
    },
    Snippet {
        label: "for",
        detail: "For-in loop",
        insert_text: "for (${1:item} in ${2:items}) {\n\t$0\n}",
    },
    Snippet {
        label: "function",
        detail: "Function declaration",
        insert_text: "function ${1:name}(${2:params}) {\n\t$0\n}",
    },
    Snippet {
        label: "match",
        detail: "Match expression",
        insert_text: "match (${1:value}) {\n\t${2:pattern} => $0\n}",
    },
    Snippet {
        label: "switch",
        detail: "Switch statement",
        insert_text:
            "switch (${1:value}) {\n\tcase ${2:pattern}:\n\t\t$0\n\tdefault:\n\t\tbreak\n}",
    },
    Snippet {
        label: "try-catch",
        detail: "Try-catch statement",
        insert_text: "try {\n\t$1\n} catch (${2:e}) {\n\t$0\n}",
    },
    Snippet {
        label: "async-function",
        detail: "Async function declaration",
        insert_text: "async function ${1:name}(${2:params}) {\n\t$0\n}",
    },
    Snippet {
        label: "step",
        detail: "Loop engineering step block",
        insert_text: "step ${1:name} {\n\t$0\n}",
    },
    Snippet {
        label: "gate",
        detail: "Loop engineering gate block",
        insert_text: "gate ${1:name} {\n\t$0\n}",
    },
    Snippet {
        label: "chain",
        detail: "Loop engineering chain block",
        insert_text: "chain ${1:name} {\n\tattach ${2:stepName}\n\t$0\n}",
    },
];

pub const STATEMENT_SNIPPETS_TR: &[Snippet] = &[
    Snippet {
        label: "eğer",
        detail: "Eğer deyimi",
        insert_text: "eğer (${1:koşul}) {\n\t$0\n}",
    },
    Snippet {
        label: "eğer-değilse",
        detail: "Eğer-değilse deyimi",
        insert_text: "eğer (${1:koşul}) {\n\t$2\n} değilse {\n\t$0\n}",
    },
    Snippet {
        label: "değilse-ama",
        detail: "Değilse ama (else-if) merdiveni",
        insert_text: "değilse ama (${1:koşul}) {\n\t$0\n}",
    },
    Snippet {
        label: "iken",
        detail: "iken döngüsü",
        insert_text: "iken (${1:koşul}) {\n\t$0\n}",
    },
    Snippet {
        label: "döngü",
        detail: "döngü (for-in)",
        insert_text: "döngü (${1:öğe} içinde ${2:koleksiyon}) {\n\t$0\n}",
    },
    Snippet {
        label: "işlev",
        detail: "işlev tanımı",
        insert_text: "işlev ${1:ad}(${2:parametreler}) {\n\t$0\n}",
    },
    Snippet {
        label: "eşle",
        detail: "eşle ifadesi",
        insert_text: "eşle (${1:değer}) {\n\t${2:kalıp} => $0\n}",
    },
    Snippet {
        label: "seç",
        detail: "seç deyimi",
        insert_text: "seç (${1:değer}) {\n\tdurum ${2:kalıp}:\n\t\t$0\n\tvarsayılan:\n\t\tkır\n}",
    },
    Snippet {
        label: "dene-yakala",
        detail: "Dene-yakala deyimi",
        insert_text: "dene {\n\t$1\n} yakala (${2:e}) {\n\t$0\n}",
    },
    Snippet {
        label: "eşzamansız-işlev",
        detail: "Eşzamansız işlev tanımı",
        insert_text: "eşzamansız işlev ${1:ad}(${2:parametreler}) {\n\t$0\n}",
    },
    Snippet {
        label: "adım",
        detail: "Döngü mühendisliği adım bloğu",
        insert_text: "adım ${1:isim} {\n\t$0\n}",
    },
    Snippet {
        label: "kapı",
        detail: "Döngü mühendisliği kapı bloğu",
        insert_text: "kapı ${1:isim} {\n\t$0\n}",
    },
    Snippet {
        label: "zincir",
        detail: "Döngü mühendisliği zincir bloğu",
        insert_text: "zincir ${1:isim} {\n\tekla ${2:adımAdı}\n\t$0\n}",
    },
];

pub const STATEMENT_SNIPPETS_AR: &[Snippet] = &[
    Snippet {
        label: "إذا",
        detail: "عبارة شرطية",
        insert_text: "إذا (${1:الشرط}) {\n\t$0\n}",
    },
    Snippet {
        label: "إذا-وإلا",
        detail: "عبارة إذا/وإلا",
        insert_text: "إذا (${1:الشرط}) {\n\t$2\n} وإلا {\n\t$0\n}",
    },
    Snippet {
        label: "وإلا-إذا",
        detail: "وإلا إذا",
        insert_text: "وإلا إذا (${1:الشرط}) {\n\t$0\n}",
    },
    Snippet {
        label: "بينما",
        detail: "حلقة بينما",
        insert_text: "بينما (${1:الشرط}) {\n\t$0\n}",
    },
    Snippet {
        label: "لكل",
        detail: "حلقة لكل",
        insert_text: "لكل (${1:عنصر} في ${2:المجموعة}) {\n\t$0\n}",
    },
    Snippet {
        label: "دالة",
        detail: "تعريف دالة",
        insert_text: "دالة ${1:الاسم}(${2:المعاملات}) {\n\t$0\n}",
    },
    Snippet {
        label: "اختر",
        detail: "عبارة اختر",
        insert_text: "اختر (${1:القيمة}) {\n\tحالة ${2:النمط}:\n\t\t$0\n\tافتراضي:\n\t\tاكسر\n}",
    },
    Snippet {
        label: "حاول-امسك",
        detail: "عبارة حاول/امسك",
        insert_text: "حاول {\n\t$1\n} امسك (${2:e}) {\n\t$0\n}",
    },
    Snippet {
        label: "غير-متزامن",
        detail: "تعريف دالة غير متزامنة",
        insert_text: "غير_متزامن دالة ${1:الاسم}(${2:المعاملات}) {\n\t$0\n}",
    },
    Snippet {
        label: "خطوة",
        detail: "كتلة خطوة هندسة الحلقات",
        insert_text: "خطوة ${1:الاسم} {\n\t$0\n}",
    },
    Snippet {
        label: "بوابة",
        detail: "كتلة بوابة هندسة الحلقات",
        insert_text: "بوابة ${1:الاسم} {\n\t$0\n}",
    },
    Snippet {
        label: "سلسلة",
        detail: "كتلة سلسلة هندسة الحلقات",
        insert_text: "سلسلة ${1:الاسم} {\n\tأرفق ${2:اسم_الخطوة}\n\t$0\n}",
    },
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// All top-level declaration snippets.
pub const DECLARATION_SNIPPETS: &[&[Snippet]] = &[
    DECLARATION_SNIPPETS_EN,
    DECLARATION_SNIPPETS_TR,
    DECLARATION_SNIPPETS_AR,
];

/// All statement-level snippets.
pub const STATEMENT_SNIPPETS: &[&[Snippet]] = &[
    STATEMENT_SNIPPETS_EN,
    STATEMENT_SNIPPETS_TR,
    STATEMENT_SNIPPETS_AR,
];
