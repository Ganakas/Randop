use windows::Win32::Globalization::GetUserDefaultUILanguage;

pub const CODES: [&str; 19] = [
    "en", "zh", "hi", "es", "fr", "ar", "bn", "pt", "ru", "ja", "de", "ko", "tr", "it", "vi", "id",
    "pl", "uk", "ur",
];

pub fn known(code: &str) -> Option<&'static str> {
    let primary = code.trim().split(['-', '_']).next().unwrap_or("");
    CODES
        .iter()
        .copied()
        .find(|item| item.eq_ignore_ascii_case(primary))
}

pub fn system() -> &'static str {
    let id = unsafe { GetUserDefaultUILanguage() };
    match id & 0x3FF {
        0x04 => "zh",
        0x39 => "hi",
        0x0A => "es",
        0x0C => "fr",
        0x01 => "ar",
        0x45 => "bn",
        0x16 => "pt",
        0x19 => "ru",
        0x11 => "ja",
        0x07 => "de",
        0x12 => "ko",
        0x1F => "tr",
        0x10 => "it",
        0x2A => "vi",
        0x21 => "id",
        0x15 => "pl",
        0x22 => "uk",
        0x20 => "ur",
        _ => "en",
    }
}

pub fn active() -> &'static str {
    crate::config::load()
        .lang
        .as_deref()
        .and_then(known)
        .unwrap_or_else(system)
}

fn slot(code: &str) -> usize {
    known(code).and_then(|code| CODES.iter().position(|item| *item == code)).unwrap_or(0)
}

macro_rules! row {
    ($slot:expr, $($item:literal),+ $(,)?) => {{
        const ROWS: [&str; 19] = [$($item),+];
        ROWS[$slot]
    }};
}

pub fn verb(code: &str, action: &str) -> &'static str {
    let slot = slot(code);
    match action {
        "open" => row!(
            slot,
            "Open a random item",
            "打开随机项",
            "यादृच्छिक खोलें",
            "Abrir uno al azar",
            "Ouvrir au hasard",
            "فتح عنصر عشوائي",
            "এলোমেলো খুলুন",
            "Abrir um aleatório",
            "Открыть случайный",
            "ランダムに開く",
            "Zufällig öffnen",
            "무작위로 열기",
            "Rastgele birini aç",
            "Apri uno a caso",
            "Mở mục ngẫu nhiên",
            "Buka yang acak",
            "Otwórz losowy",
            "Відкрити випадковий",
            "اتفاقی کھولیں",
        ),
        "multi" => row!(
            slot,
            "Pick several at random",
            "随机选择多个",
            "कई यादृच्छिक चुनें",
            "Elegir varios al azar",
            "En choisir plusieurs",
            "اختيار عدة عشوائيًا",
            "কয়েকটি এলোমেলো বাছুন",
            "Escolher vários",
            "Выбрать несколько случайных",
            "いくつかをランダムに選ぶ",
            "Mehrere zufällig wählen",
            "여러 개 무작위 선택",
            "Birkaçını rastgele seç",
            "Scegline alcuni a caso",
            "Chọn vài mục ngẫu nhiên",
            "Pilih beberapa acak",
            "Wybierz kilka losowych",
            "Вибрати кілька випадкових",
            "چند اتفاقی چنیں",
        ),
        _ => row!(
            slot,
            "Random",
            "随机",
            "यादृच्छिक",
            "Al azar",
            "Au hasard",
            "عشوائي",
            "এলোমেলো",
            "Aleatório",
            "Случайно",
            "ランダム",
            "Zufällig",
            "무작위",
            "Rastgele",
            "A caso",
            "Ngẫu nhiên",
            "Acak",
            "Losowo",
            "Випадково",
            "اتفاقی",
        ),
    }
}

fn say(action: fn(usize) -> &'static str) -> String {
    action(slot(active())).to_string()
}

fn fill(template: &str, pairs: &[(&str, &str)]) -> String {
    let mut text = template.to_string();
    for (key, value) in pairs {
        text = text.replace(key, value);
    }
    text
}

pub fn empty() -> String {
    say(|slot| {
        row!(
            slot,
            "Nothing in this folder to pick.",
            "这个文件夹里没有可选的内容。",
            "इस फ़ोल्डर में चुनने के लिए कुछ नहीं है।",
            "No hay nada que elegir en esta carpeta.",
            "Rien à choisir dans ce dossier.",
            "لا يوجد شيء لاختياره في هذا المجلد.",
            "এই ফোল্ডারে বেছে নেওয়ার কিছু নেই।",
            "Não há nada para escolher nesta pasta.",
            "В этой папке нечего выбирать.",
            "このフォルダーには選ぶものがありません。",
            "In diesem Ordner gibt es nichts zu wählen.",
            "이 폴더에서 고를 것이 없습니다.",
            "Bu klasörde seçilecek bir şey yok.",
            "In questa cartella non c'è niente da scegliere.",
            "Thư mục này không có gì để chọn.",
            "Tidak ada yang bisa dipilih di folder ini.",
            "W tym folderze nie ma czego wybrać.",
            "У цій папці нічого вибирати.",
            "اس فولڈر میں چننے کے لیے کچھ نہیں۔",
        )
    })
}

pub fn folder_missing(path: &str) -> String {
    let template = say(|slot| {
        row!(
            slot,
            "Folder not found:\n{path}",
            "找不到文件夹：\n{path}",
            "फ़ोल्डर नहीं मिला:\n{path}",
            "Carpeta no encontrada:\n{path}",
            "Dossier introuvable :\n{path}",
            "المجلد غير موجود:\n{path}",
            "ফোল্ডার পাওয়া যায়নি:\n{path}",
            "Pasta não encontrada:\n{path}",
            "папка не найдена:\n{path}",
            "フォルダーが見つかりません:\n{path}",
            "Ordner nicht gefunden:\n{path}",
            "폴더를 찾을 수 없습니다:\n{path}",
            "Klasör bulunamadı:\n{path}",
            "Cartella non trovata:\n{path}",
            "Không tìm thấy thư mục:\n{path}",
            "Folder tidak ditemukan:\n{path}",
            "Nie znaleziono folderu:\n{path}",
            "папку не знайдено:\n{path}",
            "فولڈر نہیں ملا:\n{path}",
        )
    });
    fill(&template, &[("{path}", path)])
}

pub fn bad_command() -> String {
    say(|slot| {
        row!(
            slot,
            "Didn't understand the command.",
            "无法识别这个命令。",
            "कमांड समझ नहीं आया।",
            "No entendí el comando.",
            "Commande incompréhensible.",
            "تعذر فهم الأمر.",
            "কমান্ড বোঝা যায়নি।",
            "Não entendi o comando.",
            "не понял команду",
            "コマンドを理解できませんでした。",
            "Der Befehl war unverständlich.",
            "명령을 이해하지 못했습니다.",
            "Komut anlaşılamadı.",
            "Comando non capito.",
            "Không hiểu lệnh.",
            "Perintah tidak dimengerti.",
            "Nie rozumiem polecenia.",
            "не зрозумів команду",
            "کمانڈ سمجھ نہیں آئی۔",
        )
    })
}

pub fn no_folder() -> String {
    say(|slot| {
        row!(
            slot,
            "Explorer didn't pass a folder.",
            "资源管理器没有传入文件夹。",
            "एक्सप्लोरर ने फ़ोल्डर नहीं दिया।",
            "El Explorador no pasó una carpeta.",
            "L'Explorateur n'a pas transmis de dossier.",
            "لم يمرّر المستكشف مجلدًا.",
            "এক্সপ্লোরার কোনো ফোল্ডার দেয়নি।",
            "O Explorador não passou uma pasta.",
            "проводник не передал папку",
            "エクスプローラーがフォルダーを渡しませんでした。",
            "Der Explorer hat keinen Ordner übergeben.",
            "탐색기가 폴더를 넘기지 않았습니다.",
            "Gezgin bir klasör iletmedi.",
            "Esplora risorse non ha passato una cartella.",
            "Explorer không truyền thư mục.",
            "Explorer tidak mengirim folder.",
            "Eksplorator nie przekazał folderu.",
            "провідник не передав папку",
            "ایکسپلورر نے فولڈر نہیں دیا۔",
        )
    })
}

pub fn save_failed(err: &str) -> String {
    with_err(
        |slot| {
            row!(
                slot,
                "Couldn't save settings: {err}",
                "无法保存设置：{err}",
                "सेटिंग सहेजी नहीं जा सकी: {err}",
                "No se pudo guardar la configuración: {err}",
                "Impossible d'enregistrer les réglages : {err}",
                "تعذر حفظ الإعدادات: {err}",
                "সেটিংস সেভ করা যায়নি: {err}",
                "Não foi possível guardar as definições: {err}",
                "не сохранить настройки: {err}",
                "設定を保存できません: {err}",
                "Einstellungen konnten nicht gespeichert werden: {err}",
                "설정을 저장할 수 없습니다: {err}",
                "Ayarlar kaydedilemedi: {err}",
                "Impossibile salvare le impostazioni: {err}",
                "Không lưu được cài đặt: {err}",
                "Tidak bisa menyimpan pengaturan: {err}",
                "Nie udało się zapisać ustawień: {err}",
                "не зберегти налаштування: {err}",
                "ترتیبات محفوظ نہیں ہو سکیں: {err}",
            )
        },
        err,
    )
}

pub fn write_menu(err: &str) -> String {
    with_err(
        |slot| {
            row!(
                slot,
                "Couldn't write the menu: {err}",
                "无法写入菜单：{err}",
                "मेनू लिखा नहीं जा सका: {err}",
                "No se pudo escribir el menú: {err}",
                "Impossible d'écrire le menu : {err}",
                "تعذر كتابة القائمة: {err}",
                "মেনু লেখা যায়নি: {err}",
                "Não foi possível gravar o menu: {err}",
                "не записать меню: {err}",
                "メニューを書き込めません: {err}",
                "Menü konnte nicht geschrieben werden: {err}",
                "메뉴를 쓸 수 없습니다: {err}",
                "Menü yazılamadı: {err}",
                "Impossibile scrivere il menu: {err}",
                "Không ghi được menu: {err}",
                "Tidak bisa menulis menu: {err}",
                "Nie udało się zapisać menu: {err}",
                "не записати меню: {err}",
                "مینو لکھا نہیں جا سکا: {err}",
            )
        },
        err,
    )
}

pub fn remove_menu(err: &str) -> String {
    with_err(
        |slot| {
            row!(
                slot,
                "Couldn't remove the menu: {err}",
                "无法移除菜单：{err}",
                "मेनू हटाया नहीं जा सका: {err}",
                "No se pudo quitar el menú: {err}",
                "Impossible de retirer le menu : {err}",
                "تعذر إزالة القائمة: {err}",
                "মেনু সরানো যায়নি: {err}",
                "Não foi possível remover o menu: {err}",
                "не убрать меню: {err}",
                "メニューを削除できません: {err}",
                "Menü konnte nicht entfernt werden: {err}",
                "메뉴를 제거할 수 없습니다: {err}",
                "Menü kaldırılamadı: {err}",
                "Impossibile rimuovere il menu: {err}",
                "Không gỡ được menu: {err}",
                "Tidak bisa menghapus menu: {err}",
                "Nie udało się usunąć menu: {err}",
                "не прибрати меню: {err}",
                "مینو ہٹایا نہیں جا سکا: {err}",
            )
        },
        err,
    )
}

pub fn open_link() -> String {
    say(|slot| {
        row!(
            slot,
            "Couldn't open the link.",
            "无法打开链接。",
            "लिंक नहीं खुल सका।",
            "No se pudo abrir el enlace.",
            "Impossible d'ouvrir le lien.",
            "تعذر فتح الرابط.",
            "লিংক খোলা যায়নি।",
            "Não foi possível abrir a ligação.",
            "не получилось открыть ссылку",
            "リンクを開けませんでした。",
            "Link konnte nicht geöffnet werden.",
            "링크를 열 수 없습니다.",
            "Bağlantı açılamadı.",
            "Impossibile aprire il link.",
            "Không mở được liên kết.",
            "Tidak bisa membuka tautan.",
            "Nie udało się otworzyć linku.",
            "не вдалося відкрити посилання",
            "لنک نہیں کھل سکا۔",
        )
    })
}

pub fn open_file(name: &str, code: &str) -> String {
    let template = say(|slot| {
        row!(
            slot,
            "Couldn't open \"{name}\" (code {code}).",
            "无法打开「{name}」（代码 {code}）。",
            "\"{name}\" नहीं खुल सका (कोड {code})।",
            "No se pudo abrir \"{name}\" (código {code}).",
            "Impossible d'ouvrir « {name} » (code {code}).",
            "تعذر فتح \"{name}\" (الرمز {code}).",
            "\"{name}\" খোলা যায়নি (কোড {code})।",
            "Não foi possível abrir \"{name}\" (código {code}).",
            "не получилось открыть «{name}» (код {code})",
            "「{name}」を開けませんでした（コード {code}）。",
            "„{name}“ konnte nicht geöffnet werden (Code {code}).",
            "\"{name}\"을(를) 열 수 없습니다(코드 {code}).",
            "\"{name}\" açılamadı (kod {code}).",
            "Impossibile aprire \"{name}\" (codice {code}).",
            "Không mở được \"{name}\" (mã {code}).",
            "Tidak bisa membuka \"{name}\" (kode {code}).",
            "Nie udało się otworzyć „{name}” (kod {code}).",
            "не вдалося відкрити «{name}» (код {code})",
            "\"{name}\" نہیں کھل سکا (کوڈ {code})۔",
        )
    });
    fill(&template, &[("{name}", name), ("{code}", code)])
}

pub fn shell_path() -> String {
    say(|slot| {
        row!(
            slot,
            "Explorer didn't understand the path.",
            "资源管理器无法识别路径。",
            "एक्सप्लोरर पथ समझ नहीं पाया।",
            "El Explorador no entendió la ruta.",
            "L'Explorateur n'a pas compris le chemin.",
            "لم يفهم المستكشف المسار.",
            "এক্সপ্লোরার পাথ বোঝেনি।",
            "O Explorador não entendeu o caminho.",
            "проводник не понял путь",
            "エクスプローラーがパスを理解できませんでした。",
            "Der Explorer hat den Pfad nicht verstanden.",
            "탐색기가 경로를 이해하지 못했습니다.",
            "Gezgin yolu anlamadı.",
            "Esplora risorse non ha capito il percorso.",
            "Explorer không hiểu đường dẫn.",
            "Explorer tidak memahami jalur.",
            "Eksplorator nie zrozumiał ścieżki.",
            "провідник не зрозумів шлях",
            "ایکسپلورر راستہ سمجھ نہیں سکا۔",
        )
    })
}

pub fn show_file() -> String {
    say(|slot| {
        row!(
            slot,
            "Couldn't show the file.",
            "无法显示文件。",
            "फ़ाइल दिखाई नहीं जा सकी।",
            "No se pudo mostrar el archivo.",
            "Impossible d'afficher le fichier.",
            "تعذر إظهار الملف.",
            "ফাইল দেখানো যায়নি।",
            "Não foi possível mostrar o ficheiro.",
            "не получилось показать файл",
            "ファイルを表示できませんでした。",
            "Datei konnte nicht angezeigt werden.",
            "파일을 표시할 수 없습니다.",
            "Dosya gösterilemedi.",
            "Impossibile mostrare il file.",
            "Không hiện được tệp.",
            "Tidak bisa menampilkan berkas.",
            "Nie udało się pokazać pliku.",
            "не вдалося показати файл",
            "فائل نہیں دکھائی جا سکی۔",
        )
    })
}

pub fn show_failed(err: &str) -> String {
    with_err(
        |slot| {
            row!(
                slot,
                "Couldn't show the file: {err}",
                "无法显示文件：{err}",
                "फ़ाइल दिखाई नहीं जा सकी: {err}",
                "No se pudo mostrar el archivo: {err}",
                "Impossible d'afficher le fichier : {err}",
                "تعذر إظهار الملف: {err}",
                "ফাইল দেখানো যায়নি: {err}",
                "Não foi possível mostrar o ficheiro: {err}",
                "не получилось показать файл: {err}",
                "ファイルを表示できません: {err}",
                "Datei konnte nicht angezeigt werden: {err}",
                "파일을 표시할 수 없습니다: {err}",
                "Dosya gösterilemedi: {err}",
                "Impossibile mostrare il file: {err}",
                "Không hiện được tệp: {err}",
                "Tidak bisa menampilkan berkas: {err}",
                "Nie udało się pokazać pliku: {err}",
                "не вдалося показати файл: {err}",
                "فائل نہیں دکھائی جا سکی: {err}",
            )
        },
        err,
    )
}

pub fn open_explorer(err: &str) -> String {
    with_err(
        |slot| {
            row!(
                slot,
                "Couldn't open Explorer: {err}",
                "无法打开资源管理器：{err}",
                "एक्सप्लोरर नहीं खुल सका: {err}",
                "No se pudo abrir el Explorador: {err}",
                "Impossible d'ouvrir l'Explorateur : {err}",
                "تعذر فتح المستكشف: {err}",
                "এক্সপ্লোরার খোলা যায়নি: {err}",
                "Não foi possível abrir o Explorador: {err}",
                "не открыть проводник: {err}",
                "エクスプローラーを開けません: {err}",
                "Explorer konnte nicht geöffnet werden: {err}",
                "탐색기를 열 수 없습니다: {err}",
                "Gezgin açılamadı: {err}",
                "Impossibile aprire Esplora risorse: {err}",
                "Không mở được Explorer: {err}",
                "Tidak bisa membuka Explorer: {err}",
                "Nie udało się otworzyć Eksploratora: {err}",
                "не відкрити провідник: {err}",
                "ایکسپلورر نہیں کھل سکا: {err}",
            )
        },
        err,
    )
}

pub fn open_folder(err: &str) -> String {
    with_err(
        |slot| {
            row!(
                slot,
                "Couldn't open the folder: {err}",
                "无法打开文件夹：{err}",
                "फ़ोल्डर नहीं खुल सका: {err}",
                "No se pudo abrir la carpeta: {err}",
                "Impossible d'ouvrir le dossier : {err}",
                "تعذر فتح المجلد: {err}",
                "ফোল্ডার খোলা যায়নি: {err}",
                "Não foi possível abrir a pasta: {err}",
                "не открыть папку: {err}",
                "フォルダーを開けません: {err}",
                "Ordner konnte nicht geöffnet werden: {err}",
                "폴더를 열 수 없습니다: {err}",
                "Klasör açılamadı: {err}",
                "Impossibile aprire la cartella: {err}",
                "Không mở được thư mục: {err}",
                "Tidak bisa membuka folder: {err}",
                "Nie udało się otworzyć folderu: {err}",
                "не відкрити папку: {err}",
                "فولڈر نہیں کھل سکا: {err}",
            )
        },
        err,
    )
}

pub fn already_latest() -> String {
    say(|slot| {
        row!(
            slot,
            "This is already the latest version.",
            "这已是最新版本。",
            "यह पहले से नवीनतम संस्करण है।",
            "Esta ya es la última versión.",
            "C'est déjà la dernière version.",
            "هذا هو أحدث إصدار بالفعل.",
            "এটি ইতিমধ্যে সর্বশেষ সংস্করণ।",
            "Esta já é a versão mais recente.",
            "Это уже последняя версия",
            "すでに最新バージョンです。",
            "Das ist bereits die neueste Version.",
            "이미 최신 버전입니다.",
            "Bu zaten en son sürüm.",
            "Questa è già l'ultima versione.",
            "Đây đã là phiên bản mới nhất.",
            "Ini sudah versi terbaru.",
            "To już najnowsza wersja.",
            "Це вже остання версія",
            "یہ پہلے ہی تازہ ترین ورژن ہے۔",
        )
    })
}

pub fn no_asset() -> String {
    say(|slot| {
        row!(
            slot,
            "The release has no Randop.exe.",
            "发布中没有 Randop.exe。",
            "रिलीज़ में Randop.exe नहीं है।",
            "La versión no incluye Randop.exe.",
            "La version ne contient pas Randop.exe.",
            "لا يحتوي الإصدار على Randop.exe.",
            "রিলিজে Randop.exe নেই।",
            "A versão não tem Randop.exe.",
            "в релизе нет файла Randop.exe",
            "リリースに Randop.exe がありません。",
            "Die Version enthält keine Randop.exe.",
            "릴리스에 Randop.exe가 없습니다.",
            "Sürümde Randop.exe yok.",
            "La versione non contiene Randop.exe.",
            "Bản phát hành không có Randop.exe.",
            "Rilis tidak memiliki Randop.exe.",
            "W wydaniu nie ma Randop.exe.",
            "у випуску немає файлу Randop.exe",
            "رہائی میں Randop.exe نہیں۔",
        )
    })
}

pub fn bad_url() -> String {
    say(|slot| {
        row!(
            slot,
            "The update link looks wrong.",
            "更新链接看起来不正确。",
            "अपडेट लिंक गलत लग रहा है।",
            "El enlace de actualización no parece válido.",
            "Le lien de mise à jour semble incorrect.",
            "رابط التحديث يبدو غير صحيح.",
            "আপডেট লিংক সঠিক মনে হচ্ছে না।",
            "A ligação da atualização parece errada.",
            "ссылка на файл обновления выглядит неверно",
            "更新リンクが正しくないようです。",
            "Der Update-Link sieht falsch aus.",
            "업데이트 링크가 올바르지 않습니다.",
            "Güncelleme bağlantısı yanlış görünüyor.",
            "Il link dell'aggiornamento sembra errato.",
            "Liên kết cập nhật có vẻ sai.",
            "Tautan pembaruan tampak salah.",
            "Link aktualizacji wygląda źle.",
            "посилання на оновлення виглядає неправильно",
            "اپڈیٹ لنک غلط لگ رہا ہے۔",
        )
    })
}

pub fn weird_reply() -> String {
    say(|slot| {
        row!(
            slot,
            "GitHub replied with something unreadable.",
            "GitHub 的回复无法识别。",
            "GitHub का जवाब समझ नहीं आया।",
            "GitHub respondió algo ilegible.",
            "GitHub a répondu quelque chose d'illisible.",
            "رد GitHub بشيء غير مفهوم.",
            "GitHub দুর্বোধ্য উত্তর দিয়েছে।",
            "O GitHub respondeu algo ilegível.",
            "GitHub ответил непонятно",
            "GitHub の応答を読み取れませんでした。",
            "GitHub hat unleserlich geantwortet.",
            "GitHub이 읽을 수 없는 응답을 보냈습니다.",
            "GitHub anlaşılmaz bir yanıt verdi.",
            "GitHub ha risposto in modo illeggibile.",
            "GitHub trả lời không đọc được.",
            "GitHub membalas sesuatu yang tidak terbaca.",
            "GitHub odpowiedział niezrozumiale.",
            "GitHub відповів незрозуміло",
            "GitHub نے ناقابلِ فہم جواب دیا۔",
        )
    })
}

pub fn not_github() -> String {
    say(|slot| {
        row!(
            slot,
            "The file did not come from GitHub.",
            "文件不是来自 GitHub。",
            "फ़ाइल GitHub से नहीं आई।",
            "El archivo no vino de GitHub.",
            "Le fichier ne vient pas de GitHub.",
            "الملف لم يأتِ من GitHub.",
            "ফাইলটি GitHub থেকে আসেনি।",
            "O ficheiro não veio do GitHub.",
            "Файл пришёл не с GitHub",
            "ファイルは GitHub からのものではありません。",
            "Die Datei kam nicht von GitHub.",
            "파일이 GitHub에서 오지 않았습니다.",
            "Dosya GitHub'dan gelmedi.",
            "Il file non proviene da GitHub.",
            "Tệp không đến từ GitHub.",
            "Berkas tidak berasal dari GitHub.",
            "Plik nie pochodzi z GitHuba.",
            "Файл надійшов не з GitHub",
            "فائل GitHub سے نہیں آئی۔",
        )
    })
}

pub fn no_release() -> String {
    say(|slot| {
        row!(
            slot,
            "No published version yet.",
            "还没有公开发布的版本。",
            "अभी कोई प्रकाशित संस्करण नहीं है।",
            "Todavía no hay una versión publicada.",
            "Pas encore de version publiée.",
            "لا توجد نسخة منشورة بعد.",
            "এখনও কোনো প্রকাশিত সংস্করণ নেই।",
            "Ainda não há uma versão publicada.",
            "Пока нет опубликованной версии",
            "まだ公開されたバージョンはありません。",
            "Noch keine veröffentlichte Version.",
            "아직 공개된 버전이 없습니다.",
            "Henüz yayımlanmış bir sürüm yok.",
            "Non c'è ancora una versione pubblicata.",
            "Chưa có phiên bản được phát hành.",
            "Belum ada versi yang diterbitkan.",
            "Nie ma jeszcze opublikowanej wersji.",
            "Поки немає опублікованої версії",
            "ابھی کوئی شائع شدہ ورژن نہیں۔",
        )
    })
}

pub fn github_refused() -> String {
    say(|slot| {
        row!(
            slot,
            "GitHub didn't provide the update.",
            "GitHub 没有提供更新。",
            "GitHub ने अपडेट नहीं दिया।",
            "GitHub no entregó la actualización.",
            "GitHub n'a pas fourni la mise à jour.",
            "لم يقدّم GitHub التحديث.",
            "GitHub আপডেট দেয়নি।",
            "O GitHub não entregou a atualização.",
            "GitHub не отдал обновление",
            "GitHub が更新を渡しませんでした。",
            "GitHub hat das Update nicht geliefert.",
            "GitHub이 업데이트를 주지 않았습니다.",
            "GitHub güncellemeyi vermedi.",
            "GitHub non ha fornito l'aggiornamento.",
            "GitHub không đưa bản cập nhật.",
            "GitHub tidak memberikan pembaruan.",
            "GitHub nie przekazał aktualizacji.",
            "GitHub не віддав оновлення",
            "GitHub نے اپڈیٹ نہیں دیا۔",
        )
    })
}

pub fn too_big() -> String {
    say(|slot| {
        row!(
            slot,
            "The update file is too large.",
            "更新文件太大。",
            "अपडेट फ़ाइल बहुत बड़ी है।",
            "El archivo de actualización es demasiado grande.",
            "Le fichier de mise à jour est trop gros.",
            "ملف التحديث كبير جدًا.",
            "আপডেট ফাইলটি অনেক বড়।",
            "O ficheiro de atualização é grande demais.",
            "Файл обновления слишком большой",
            "更新ファイルが大きすぎます。",
            "Die Update-Datei ist zu groß.",
            "업데이트 파일이 너무 큽니다.",
            "Güncelleme dosyası çok büyük.",
            "Il file di aggiornamento è troppo grande.",
            "Tệp cập nhật quá lớn.",
            "Berkas pembaruan terlalu besar.",
            "Plik aktualizacji jest za duży.",
            "Файл оновлення завеликий",
            "اپڈیٹ فائل بہت بڑی ہے۔",
        )
    })
}

pub fn not_program() -> String {
    say(|slot| {
        row!(
            slot,
            "The downloaded file doesn't look like the program.",
            "下载的文件不像是程序。",
            "डाउनलोड की गई फ़ाइल प्रोग्राम जैसी नहीं है।",
            "El archivo descargado no parece el programa.",
            "Le fichier téléchargé ne ressemble pas au programme.",
            "الملف المنزل لا يبدو البرنامج.",
            "ডাউনলোড করা ফাইলটি প্রোগ্রামের মতো নয়।",
            "O ficheiro transferido não parece o programa.",
            "Скачанный файл не похож на программу",
            "ダウンロードしたファイルはプログラムではないようです。",
            "Die heruntergeladene Datei sieht nicht nach dem Programm aus.",
            "다운로드한 파일이 프로그램으로 보이지 않습니다.",
            "İndirilen dosya programa benzemiyor.",
            "Il file scaricato non sembra il programma.",
            "Tệp đã tải không giống chương trình.",
            "Berkas yang diunduh tidak seperti programnya.",
            "Pobrany plik nie wygląda na program.",
            "Завантажений файл не схожий на програму",
            "ڈاؤن لوڈ فائل پروگرام جیسی نہیں۔",
        )
    })
}

pub fn no_self() -> String {
    say(|slot| {
        row!(
            slot,
            "Can't see its own file.",
            "找不到自己的文件。",
            "अपनी फ़ाइल नहीं दिख रही।",
            "No veo su propio archivo.",
            "Impossible de voir son propre fichier.",
            "لا يمكن رؤية ملفه.",
            "নিজের ফাইল দেখা যাচ্ছে না।",
            "Não consigo ver o próprio ficheiro.",
            "Не вижу свой файл",
            "自分のファイルが見つかりません。",
            "Die eigene Datei ist nicht zu sehen.",
            "자기 파일을 찾을 수 없습니다.",
            "Kendi dosyası görünmüyor.",
            "Non vedo il proprio file.",
            "Không thấy tệp của chính nó.",
            "Tidak melihat berkasnya sendiri.",
            "Nie widzę własnego pliku.",
            "Не бачу свій файл",
            "اپنی فائل نظر نہیں آ رہی۔",
        )
    })
}

pub fn no_dir() -> String {
    say(|slot| {
        row!(
            slot,
            "Can't see the program's folder.",
            "找不到程序所在的文件夹。",
            "प्रोग्राम का फ़ोल्डर नहीं दिख रहा।",
            "No veo la carpeta del programa.",
            "Impossible de voir le dossier du programme.",
            "لا يمكن رؤية مجلد البرنامج.",
            "প্রোগ্রামের ফোল্ডার দেখা যাচ্ছে না।",
            "Não consigo ver a pasta do programa.",
            "Не вижу папку программы",
            "プログラムのフォルダーが見つかりません。",
            "Der Ordner des Programms ist nicht zu sehen.",
            "프로그램 폴더를 찾을 수 없습니다.",
            "Programın klasörü görünmüyor.",
            "Non vedo la cartella del programma.",
            "Không thấy thư mục của chương trình.",
            "Tidak melihat folder program.",
            "Nie widzę folderu programu.",
            "Не бачу папку програми",
            "پروگرام کا فولڈر نظر نہیں آ رہا۔",
        )
    })
}

pub fn cant_write() -> String {
    say(|slot| {
        row!(
            slot,
            "Can't write the update next to the program. Put it in a folder of your own.",
            "无法把更新写到程序旁边。请把它放到你自己的文件夹里。",
            "अपडेट प्रोग्राम के बगल में नहीं लिख सकता। इसे अपने फ़ोल्डर में रखें।",
            "No puedo guardar la actualización junto al programa. Ponlo en una carpeta tuya.",
            "Impossible d'écrire la mise à jour à côté du programme. Placez-le dans un dossier à vous.",
            "لا يمكن كتابة التحديث بجانب البرنامج. ضعه في مجلد خاص بك.",
            "প্রোগ্রামের পাশে আপডেট লেখা যাচ্ছে না। এটি নিজের ফোল্ডারে রাখুন।",
            "Não consigo gravar a atualização ao lado do programa. Coloque-o numa pasta sua.",
            "Не могу записать обновление рядом с программой. Положите её в свою папку.",
            "更新をプログラムの隣に書き込めません。自分のフォルダーに置いてください。",
            "Das Update kann nicht neben dem Programm gespeichert werden. Lege es in einen eigenen Ordner.",
            "프로그램 옆에 업데이트를 쓸 수 없습니다. 자신의 폴더에 두세요.",
            "Güncelleme programın yanına yazılamıyor. Onu kendi klasörüne koy.",
            "Non posso scrivere l'aggiornamento accanto al programma. Mettilo in una tua cartella.",
            "Không ghi được bản cập nhật cạnh chương trình. Hãy đặt nó vào thư mục của bạn.",
            "Tidak bisa menulis pembaruan di samping program. Taruh di folder milikmu.",
            "Nie mogę zapisać aktualizacji obok programu. Połóż go we własnym folderze.",
            "Не можу записати оновлення поруч із програмою. Покладіть її у свою папку.",
            "اپڈیٹ پروگرام کے ساتھ نہیں لکھ سکتا۔ اسے اپنے فولڈر میں رکھیں۔",
        )
    })
}

pub fn old_busy() -> String {
    say(|slot| {
        row!(
            slot,
            "The previous copy is still in use. Close Randop and try again.",
            "上一份副本仍被占用。请关闭 Randop 后再试。",
            "पिछली प्रति अभी इस्तेमाल में है। Randop बंद करके फिर कोशिश करें।",
            "La copia anterior sigue en uso. Cierra Randop e inténtalo de nuevo.",
            "L'ancienne copie est encore utilisée. Fermez Randop et réessayez.",
            "النسخة السابقة ما زالت قيد الاستخدام. أغلق Randop وحاول مرة أخرى.",
            "আগের কপি এখনও ব্যবহৃত হচ্ছে। Randop বন্ধ করে আবার চেষ্টা করুন।",
            "A cópia anterior ainda está em uso. Feche o Randop e tente de novo.",
            "Прежняя копия ещё занята. Закройте Randop и попробуйте снова.",
            "前のコピーがまだ使用中です。Randop を閉じてからやり直してください。",
            "Die vorherige Kopie wird noch verwendet. Schließe Randop und versuche es erneut.",
            "이전 복사본이 아직 사용 중입니다. Randop을 닫고 다시 시도하세요.",
            "Önceki kopya hâlâ kullanımda. Randop'u kapatıp yeniden dene.",
            "La copia precedente è ancora in uso. Chiudi Randop e riprova.",
            "Bản sao trước vẫn đang được dùng. Hãy đóng Randop rồi thử lại.",
            "Salinan sebelumnya masih dipakai. Tutup Randop dan coba lagi.",
            "Poprzednia kopia jest nadal używana. Zamknij Randop i spróbuj ponownie.",
            "Попередня копія ще зайнята. Закрийте Randop і спробуйте знову.",
            "پچھلی کاپی ابھی استعمال میں ہے۔ Randop بند کر کے دوبارہ کوشش کریں۔",
        )
    })
}

pub fn replace_failed(err: &str) -> String {
    with_err(
        |slot| {
            row!(
                slot,
                "Couldn't replace the program: {err}",
                "无法替换程序：{err}",
                "प्रोग्राम बदला नहीं जा सका: {err}",
                "No se pudo reemplazar el programa: {err}",
                "Impossible de remplacer le programme : {err}",
                "تعذر استبدال البرنامج: {err}",
                "প্রোগ্রাম প্রতিস্থাপন করা যায়নি: {err}",
                "Não foi possível substituir o programa: {err}",
                "Не получилось заменить программу: {err}",
                "プログラムを置き換えられません: {err}",
                "Programm konnte nicht ersetzt werden: {err}",
                "프로그램을 바꿀 수 없습니다: {err}",
                "Program değiştirilemedi: {err}",
                "Impossibile sostituire il programma: {err}",
                "Không thay được chương trình: {err}",
                "Tidak bisa mengganti program: {err}",
                "Nie udało się podmienić programu: {err}",
                "Не вдалося замінити програму: {err}",
                "پروگرام تبدیل نہیں ہو سکا: {err}",
            )
        },
        err,
    )
}

pub fn replace_left_old() -> String {
    say(|slot| {
        row!(
            slot,
            "Couldn't replace the program. The working copy is still in the .old file.",
            "无法替换程序。可用的副本留在 .old 文件里。",
            "प्रोग्राम बदला नहीं जा सका। काम करने वाली प्रति .old फ़ाइल में है।",
            "No se pudo reemplazar el programa. La copia que funciona sigue en el archivo .old.",
            "Impossible de remplacer le programme. La copie utilisable est restée dans le fichier .old.",
            "تعذر استبدال البرنامج. النسخة العاملة ما زالت في ملف .old.",
            "প্রোগ্রাম প্রতিস্থাপন করা যায়নি। কাজ করা কপি .old ফাইলে আছে।",
            "Não foi possível substituir o programa. A cópia que funciona ficou no ficheiro .old.",
            "Не получилось заменить программу. Рабочая копия осталась в файле .old",
            "プログラムを置き換えられませんでした。動くコピーは .old ファイルに残っています。",
            "Programm konnte nicht ersetzt werden. Die lauffähige Kopie liegt noch in der .old-Datei.",
            "프로그램을 바꿀 수 없습니다. 동작하는 복사본은 .old 파일에 남아 있습니다.",
            "Program değiştirilemedi. Çalışan kopya .old dosyasında duruyor.",
            "Impossibile sostituire il programma. La copia funzionante è rimasta nel file .old.",
            "Không thay được chương trình. Bản còn chạy nằm trong tệp .old.",
            "Tidak bisa mengganti program. Salinan yang jalan masih di berkas .old.",
            "Nie udało się podmienić programu. Działająca kopia została w pliku .old.",
            "Не вдалося замінити програму. Робоча копія лишилася у файлі .old",
            "پروگرام تبدیل نہیں ہو سکا۔ کام کرنے والی کاپی .old فائل میں ہے۔",
        )
    })
}

pub fn new_failed(err: &str) -> String {
    with_err(
        |slot| {
            row!(
                slot,
                "The new version didn't start: {err}",
                "新版本没有启动：{err}",
                "नया संस्करण शुरू नहीं हुआ: {err}",
                "La nueva versión no arrancó: {err}",
                "La nouvelle version n'a pas démarré : {err}",
                "لم يبدأ الإصدار الجديد: {err}",
                "নতুন সংস্করণ চালু হয়নি: {err}",
                "A nova versão não iniciou: {err}",
                "Новая версия не запустилась: {err}",
                "新しいバージョンが起動しませんでした: {err}",
                "Die neue Version ist nicht gestartet: {err}",
                "새 버전이 시작되지 않았습니다: {err}",
                "Yeni sürüm başlamadı: {err}",
                "La nuova versione non si è avviata: {err}",
                "Phiên bản mới không khởi động: {err}",
                "Versi baru tidak mulai: {err}",
                "Nowa wersja nie wystartowała: {err}",
                "Нова версія не запустилася: {err}",
                "نیا ورژن شروع نہیں ہوا: {err}",
            )
        },
        err,
    )
}

pub fn new_failed_restore() -> String {
    say(|slot| {
        row!(
            slot,
            "The new version didn't start. If the window won't open, rename the .old file back.",
            "新版本没有启动。如果窗口打不开，请把 .old 文件改回原名。",
            "नया संस्करण शुरू नहीं हुआ। अगर विंडो नहीं खुलती, .old फ़ाइल का नाम वापस बदल दें।",
            "La nueva versión no arrancó. Si la ventana no se abre, vuelve a nombrar el archivo .old.",
            "La nouvelle version n'a pas démarré. Si la fenêtre ne s'ouvre pas, renommez le fichier .old.",
            "لم يبدأ الإصدار الجديد. إذا لم تفتح النافذة، أعد تسمية ملف .old.",
            "নতুন সংস্করণ চালু হয়নি। উইন্ডো না খুললে .old ফাইলের নাম ফিরিয়ে দিন।",
            "A nova versão não iniciou. Se a janela não abrir, renomeie o ficheiro .old de volta.",
            "Новая версия не запустилась. Если окно не открывается, переименуйте файл .old обратно.",
            "新しいバージョンが起動しませんでした。ウィンドウが開かないときは .old ファイルの名前を戻してください。",
            "Die neue Version ist nicht gestartet. Wenn das Fenster nicht aufgeht, benenne die .old-Datei zurück.",
            "새 버전이 시작되지 않았습니다. 창이 열리지 않으면 .old 파일 이름을 되돌리세요.",
            "Yeni sürüm başlamadı. Pencere açılmazsa .old dosyasının adını geri çevir.",
            "La nuova versione non si è avviata. Se la finestra non si apre, rinomina il file .old.",
            "Phiên bản mới không khởi động. Nếu cửa sổ không mở, hãy đổi tên tệp .old lại.",
            "Versi baru tidak mulai. Jika jendela tidak terbuka, kembalikan nama berkas .old.",
            "Nowa wersja nie wystartowała. Jeśli okno się nie otwiera, zmień nazwę pliku .old z powrotem.",
            "Нова версія не запустилася. Якщо вікно не відкривається, перейменуйте файл .old назад.",
            "نیا ورژن شروع نہیں ہوا۔ اگر ونڈو نہ کھلے تو .old فائل کا نام واپس کر دیں۔",
        )
    })
}

pub fn need_https() -> String {
    say(|slot| {
        row!(
            slot,
            "The update link must be https.",
            "更新链接必须是 https。",
            "अपडेट लिंक https होना चाहिए।",
            "El enlace de actualización debe ser https.",
            "Le lien de mise à jour doit être en https.",
            "يجب أن يكون رابط التحديث https.",
            "আপডেট লিংক https হতে হবে।",
            "A ligação da atualização tem de ser https.",
            "Ссылка обновления должна быть https",
            "更新リンクは https である必要があります。",
            "Der Update-Link muss https sein.",
            "업데이트 링크는 https여야 합니다.",
            "Güncelleme bağlantısı https olmalı.",
            "Il link dell'aggiornamento deve essere https.",
            "Liên kết cập nhật phải là https.",
            "Tautan pembaruan harus https.",
            "Link aktualizacji musi być https.",
            "Посилання оновлення має бути https",
            "اپڈیٹ لنک https ہونا چاہیے۔",
        )
    })
}

pub fn offline() -> String {
    say(|slot| {
        row!(
            slot,
            "Couldn't reach GitHub.",
            "无法连接到 GitHub。",
            "GitHub तक नहीं पहुँच सका।",
            "No se pudo contactar con GitHub.",
            "Impossible de joindre GitHub.",
            "تعذر الوصول إلى GitHub.",
            "GitHub-এ পৌঁছানো যায়নি।",
            "Não foi possível contactar o GitHub.",
            "Не получилось связаться с GitHub",
            "GitHub に接続できませんでした。",
            "GitHub war nicht erreichbar.",
            "GitHub에 연결할 수 없습니다.",
            "GitHub'a ulaşılamadı.",
            "Impossibile raggiungere GitHub.",
            "Không kết nối được với GitHub.",
            "Tidak bisa menghubungi GitHub.",
            "Nie udało się połączyć z GitHubem.",
            "Не вдалося зв'язатися з GitHub",
            "GitHub تک نہیں پہنچ سکا۔",
        )
    })
}

fn with_err(template: fn(usize) -> &'static str, err: &str) -> String {
    fill(&say(template), &[("{err}", err)])
}

#[cfg(test)]
mod tests {
    use super::{known, verb};

    #[test]
    fn codes_round_trip_and_russian_menu_stays() {
        assert_eq!(known("zh-CN"), Some("zh"));
        assert_eq!(known("PT"), Some("pt"));
        assert_eq!(known("nope"), None);
        assert_eq!(verb("ru", "highlight"), "Случайно");
        assert_eq!(verb("ru", "open"), "Открыть случайный");
        assert_eq!(verb("ru", "multi"), "Выбрать несколько случайных");
        for code in super::CODES {
            for action in ["highlight", "open", "multi"] {
                assert!(!verb(code, action).is_empty());
            }
        }
    }
}
