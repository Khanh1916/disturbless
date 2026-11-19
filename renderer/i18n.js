'use strict';
// Hai ngôn ngữ giao diện: Tiếng Việt (vi) và English (en). Dùng: t('khoá', { tham_số }).
// Giá trị có thể là chuỗi chứa {tham_số}, hoặc hàm (tham_số) => chuỗi (để chia số ít/số nhiều).
(function () {
  const plural = (one, many) => (p) => `${p.n} ${Number(p.n) === 1 ? one : many}`;

  const DICT = {
    vi: {
      'app.title': 'Disturbless',
      // đơn vị
      'unit.h': 'giờ', 'unit.min': 'phút',
      'days': '{n} ngày',
      // mục tiêu
      'goal.study': 'Học bài', 'goal.study.sub': 'Ôn thi, làm bài tập, xem giáo trình',
      'goal.work': 'Làm việc', 'goal.work.sub': 'Viết, thiết kế, lập trình, báo cáo',
      'goal.read': 'Đọc sách, học ngoài máy', 'goal.read.sub': 'Khoá cả màn hình, bạn rời máy mà học',
      // nhóm chặn
      'group.social': 'Mạng xã hội', 'group.social.eg': 'Facebook, TikTok, Instagram…',
      'group.video': 'Video & nhạc giải trí', 'group.video.eg': 'YouTube, Netflix, Spotify…',
      'group.game': 'Chơi game', 'group.game.eg': 'Steam, Liên Minh, Roblox…',
      'group.chat': 'Nhắn tin & chat', 'group.chat.eg': 'Zalo, Discord, Messenger…',
      'group.shop': 'Mua sắm online', 'group.shop.eg': 'Shopee, Lazada, Tiki…',
      // mức nghiêm khắc
      'level.easy': 'Dễ tính', 'level.easy.desc': 'Chờ 30 giây rồi gõ một câu ngắn. App lạ chỉ bị “mời” đóng.', 'level.easy.brief': 'chờ 30 giây, gõ một câu ngắn',
      'level.normal': 'Vừa', 'level.normal.desc': 'Chờ 2 phút rồi gõ hai câu. App lạ bị đóng.', 'level.normal.brief': 'chờ 2 phút, gõ hai câu',
      'level.hard': 'Gắt', 'level.hard.desc': 'Chờ 5 phút rồi gõ ba câu dài. App lạ bị tắt cứng.', 'level.hard.brief': 'chờ 5 phút, gõ ba câu dài',
      // bắt đầu nhanh
      'quick.25': 'Ngắn gọn, dễ bắt đầu', 'quick.50': 'Một buổi học vừa phải', 'quick.90': 'Học sâu, có nghỉ giữa chừng',
      // lời mèo trong trình hướng dẫn
      'say.goal': 'Hôm nay bạn định làm gì thế?',
      'say.time': 'Bao lâu thì bạn trụ nổi? Đừng hứa lố nha.',
      'say.block': 'Mấy thứ hay làm bạn xao nhãng, để tui chặn giùm.',
      'say.level': 'Muốn tui hiền hay dữ khi bạn đòi bỏ cuộc?',
      'say.review': 'Chốt kèo nha? Vào rồi là hết đường lui đó.',
      // giới thiệu
      'intro.1.title': 'Xin chào, tui là mèo canh học!', 'intro.1.text': 'Bạn đặt giờ học, tui ngồi canh cho tới khi xong.',
      'intro.2.title': 'Vào phiên là không thoát dễ đâu nha', 'intro.2.text': 'Muốn bỏ giữa chừng phải chờ rồi gõ cả đoạn văn. Mở app bị cấm là tui mắng, còn tát nhẹ nữa.',
      'intro.3.title': 'Thử một phiên nhé?', 'intro.3.text': 'Chọn một mốc thời gian, bấm bắt đầu là xong. Muốn chỉnh kỹ hơn thì có sẵn tuỳ chọn.',
      'intro.skip': 'Bỏ qua', 'intro.next': 'Tiếp theo', 'intro.start': 'Bắt đầu thôi',
      // điều hướng
      'nav.home': 'Trang chủ', 'nav.stats': 'Thống kê', 'nav.settings': 'Cài đặt',
      // trang chủ
      'home.stale': 'Còn vài trang web đang bị chặn từ lần trước.', 'home.stale.btn': 'Mở lại các trang web',
      'home.h1': 'Học tập trung<br>cùng mèo nào',
      'home.lead': 'Chọn thời gian, bấm bắt đầu rồi cứ học. Mèo canh giúp bạn đến khi xong.',
      'home.custom': 'Tuỳ chỉnh từng bước', 'home.last': 'Như lần trước',
      'home.streak': '{n} ngày liên tiếp', 'home.today': 'Hôm nay: {v}', 'home.today.none': 'chưa học',
      // trình hướng dẫn
      'wiz.step': 'Bước {n}/{total}', 'wiz.home': 'Về trang chủ', 'wiz.back': 'Quay lại', 'wiz.next': 'Tiếp', 'wiz.start': 'Bắt đầu',
      'wiz.goal.q': 'Bạn định làm gì?', 'wiz.goal.s': 'Chọn một cái gần đúng nhất nhé.',
      'wiz.time.q': 'Học trong bao lâu?', 'wiz.time.s': 'Tổng thời gian bạn muốn học (giờ nghỉ nếu có sẽ cộng thêm).',
      'wiz.time.2h': '2 tiếng', 'wiz.time.2h.sub': 'Thử thách lớn', 'wiz.time.90.sub': 'Một mạch thật sâu',
      'wiz.time.other': 'Hoặc nhập số khác:', 'wiz.time.min': 'phút',
      'wiz.rest.q': 'Có muốn nghỉ giữa chừng không?',
      'wiz.rest.none': 'Học một mạch', 'wiz.rest.none.sub': 'Không nghỉ giữa chừng',
      'wiz.rest.short': 'Nghỉ ngắn', 'wiz.rest.short.sub': '25 phút học · 5 phút nghỉ',
      'wiz.rest.long': 'Nghỉ dài', 'wiz.rest.long.sub': '50 phút học · 10 phút nghỉ',
      'wiz.rest.note.rounds': 'Với {min} học, bạn sẽ học {rounds} vòng, mỗi vòng {each}. Giờ nghỉ cộng thêm bên ngoài, tổng {total}.',
      'wiz.rest.note.short': 'Thời gian hơi ngắn nên mình học một mạch luôn nha.',
      'wiz.rest.note.default': 'Lúc nghỉ, các app được mở khoá tạm thời.',
      'wiz.block.q': 'Chặn gì cho đỡ xao nhãng?', 'wiz.block.s': 'Các chương trình của Windows và tiến trình hệ thống luôn được giữ nguyên, mèo không đụng tới.',
      'wiz.block.mode.block': 'Chặn thứ gây xao nhãng', 'wiz.block.mode.allow': 'Chỉ cho dùng app mình chọn',
      'wiz.block.note.block': 'Khuyên dùng: chỉ chặn đúng những thứ bạn chọn bên dưới.', 'wiz.block.note.allow': 'Mọi app không nằm trong danh sách bạn chọn sẽ bị đóng.',
      'wiz.block.allow.q': 'Bạn cần dùng app nào để học?', 'wiz.block.allow.s': 'Bấm để chọn (trình duyệt, Word, phần mềm học…). Hãy mở sẵn các app đó trước.',
      'wiz.block.rescan': 'Quét lại app đang mở',
      'wiz.block.groups': 'Chặn nhóm nào?', 'wiz.block.groups.allow': 'Trang web vẫn bị chặn',
      'wiz.block.tabs': 'Tự đóng tab đang xem nếu là trang bị chặn', 'wiz.block.tabs.s': 'Mèo mắng rồi chỉ đóng đúng tab đó, không đóng cả trình duyệt.',
      'wiz.block.more': 'Thêm app hoặc trang web khác', 'wiz.block.addapp': 'Chặn thêm app đang mở',
      'wiz.block.sites': 'Trang web khác, mỗi dòng một địa chỉ, ví dụ: facebook.com',
      'apps.loading': 'Đang tìm các app bạn đang mở…', 'apps.none': 'Chưa thấy app nào đang mở. Hãy mở app bạn muốn rồi bấm lại nút nhé.',
      'wiz.level.q': 'Mèo nên nghiêm đến đâu?', 'wiz.level.s': 'Khi bạn đòi bỏ cuộc hoặc mở app bị cấm.',
      'wiz.review.q': 'Sẵn sàng chưa?', 'wiz.review.s': 'Kiểm tra lại một lượt nhé.',
      'sum.rounds': '{rounds} vòng × {each}, nghỉ {brk} giữa các vòng. Tính cả giờ nghỉ là {total}', 'sum.once': 'Học một mạch, không nghỉ giữa chừng',
      'sum.full': 'Khoá cả màn hình', 'sum.full.s': 'Màn hình bị che kín, bạn rời máy ra học',
      'sum.allow': 'Chỉ cho dùng {n} app bạn chọn', 'sum.allow.s': 'Mọi app khác sẽ bị đóng',
      'sum.block': 'Chặn: {names}', 'sum.block.none': 'Không chặn gì cả', 'sum.sites': '{n} trang web', 'sum.apps': ' · {n} app',
      'sum.tabs.on': 'Tự đóng tab bị cấm: Bật', 'sum.tabs.off': 'Tự đóng tab bị cấm: Tắt',
      'sum.tabs.on.s': 'Tab đang xem là trang bị cấm sẽ bị mắng rồi đóng (chỉ đóng đúng tab đó)', 'sum.tabs.off.s': 'Mèo sẽ không tự đóng tab nào',
      'sum.level': 'Mức {name}', 'sum.level.s': 'Muốn thoát sớm: {brief}',
      // hộp xác nhận
      'confirm.checking': 'Đang kiểm tra…', 'confirm.title': 'Sẵn sàng bắt đầu', 'confirm.body': '<b>{brief}</b> sẽ bắt đầu ngay khi bạn bấm nút.',
      'confirm.intent': 'Lần này bạn định làm gì?', 'confirm.optional': '(không bắt buộc)', 'confirm.intent.ph': 'Ví dụ: Ôn chương 3 môn Toán',
      'confirm.intent.hint': 'Dòng này sẽ hiện trên đồng hồ để bạn nhớ vì sao mình đang ở đây.',
      'confirm.close': 'Các app đang mở này sẽ bị đóng:', 'confirm.save': 'Nhớ lưu bài trước khi bấm nhé.',
      'confirm.uac': 'Để chặn trang web, Windows sẽ hỏi “Cho phép thay đổi?” lúc bắt đầu và lúc xong. Bạn bấm “Có” nhé.',
      'confirm.no': 'Chưa đâu', 'confirm.yes': 'Bắt đầu',
      // đang chạy
      'run.who': '{name} đang canh bạn', 'run.paused': 'Đang tạm dừng', 'run.break': 'Giờ nghỉ', 'run.focus': 'Đang tập trung',
      'run.left': 'Còn tổng {t}', 'run.tabs': 'Đang canh tab trình duyệt', 'run.viol': 'Lỡ mở app cấm {n} lần', 'run.restarts': 'Bị ngắt {n} lần',
      // thống kê
      'stats.title': 'Thống kê của bạn', 'stats.lead': 'Mỗi ngày có ít nhất một phiên hoàn thành là được tính vào chuỗi ngày liên tiếp.',
      'stats.streak': 'Chuỗi hiện tại · dài nhất {n}', 'stats.today': 'Hôm nay', 'stats.week': '7 ngày qua', 'stats.rate': 'Hoàn thành {a}/{b} phiên',
      'stats.emg': 'Khẩn cấp: {n} lần (không tính giờ, không làm mất chuỗi)',
      'stats.chart': '7 ngày gần nhất', 'stats.recent': 'Các phiên gần đây', 'stats.empty': 'Chưa có phiên nào. Bắt đầu phiên đầu tiên nhé!',
      'stats.reason': 'lý do: {r}', 'stats.focused': 'tập trung {a} / {b}', 'stats.viol': 'lỡ mở app cấm {n} lần', 'stats.paused': 'có tạm dừng',
      'stats.tag.emg': 'Khẩn cấp', 'stats.tag.done': 'Xong rồi', 'stats.tag.quit': 'Bỏ dở',
      'stats.pull': 'Điều hay kéo bạn đi', 'stats.pull.empty': 'Chưa có dữ liệu. Khi mở app hay tab bị chặn, mèo sẽ hỏi lý do và ghi lại ở đây.',
      'stats.pull.apps': 'Hay mở nhất', 'stats.pull.why': 'Vì sao', 'stats.pull.times': '{n} lần',
      // cài đặt
      'set.title': 'Cài đặt', 'set.lead': 'Chỉ đổi được khi không đang trong phiên.',
      'set.cat': 'Chú mèo', 'set.voice': 'Mèo nói thành tiếng',
      'set.voice.yes': 'Máy bạn đã có giọng đọc tiếng Việt.', 'set.voice.no': 'Máy chưa có giọng tiếng Việt nên mèo chỉ hiện chữ. Cài thêm ở Cài đặt Windows → Thời gian & ngôn ngữ → Giọng nói.',
      'set.cheer': 'Mèo động viên bao lâu một lần', 'set.cheer.s': 'Lời cổ vũ hiện ở cửa sổ nhỏ trong lúc học.',
      'set.cheer.0': 'Không cần', 'set.cheer.60': 'Mỗi 1 phút', 'set.cheer.120': 'Mỗi 2 phút', 'set.cheer.240': 'Mỗi 4 phút', 'set.cheer.600': 'Mỗi 10 phút',
      'set.scold': 'Xem thử mèo mắng', 'set.scold.s': 'Chỉ là hiệu ứng thôi, không ảnh hưởng gì đến máy bạn.',
      'set.scold.1': 'Nhắc nhở', 'set.scold.2': 'Mắng', 'set.scold.3': 'Tát mạnh',
      'set.level': 'Mức nghiêm khắc mặc định',
      'set.general': 'Chung',
      'set.auto': 'Tự mở cùng Windows', 'set.auto.s': 'Nếu máy khởi động lại giữa lúc học, phiên sẽ được tiếp tục.',
      'set.font': 'Cỡ chữ', 'set.font.s': 'Phóng to chữ và giao diện cho dễ đọc.',
      'set.font.1': 'Vừa', 'set.font.1.15': 'Lớn', 'set.font.1.3': 'Rất lớn',
      'set.lang': 'Ngôn ngữ', 'set.lang.s': 'Cả giao diện, lời mèo và đoạn văn phải gõ khi thoát sớm.',
      'set.intro': 'Xem lại phần giới thiệu', 'set.intro.s': 'Mèo sẽ chào bạn lại từ đầu.', 'set.intro.btn': 'Xem lại',
      // thoát sớm
      'exit.hint': 'Muốn bỏ dở giữa chừng? Sẽ mất streak và mèo sẽ không vui đâu.', 'exit.btn': 'Tôi muốn thoát sớm',
      'exit.wait': 'Hãy nghĩ lại. Bạn có thể tiếp tục thủ tục thoát sau {t}', 'exit.back': 'Thôi, quay lại tập trung',
      'exit.type': 'Gõ lại <b>chính xác</b> đoạn văn sau để thoát (không dán được):', 'exit.ph': 'Gõ lại đoạn văn ở đây…',
      'exit.back2': 'Quay lại tập trung', 'exit.confirm': 'Xác nhận thoát (mất streak)', 'exit.mismatch': 'Chưa khớp đoạn văn. Hãy gõ lại cho đúng.',
      // khẩn cấp
      'emg.open': 'Có việc khẩn cấp', 'emg.ask': '<b>Có chuyện gì thế?</b> Mèo không mắng đâu. Chọn lý do, lịch sử sẽ ghi lại.',
      'emg.r.health': 'Sức khoẻ, an toàn', 'emg.r.call': 'Cuộc gọi, họp đột xuất', 'emg.r.need': 'Cần trang web hoặc app đang bị chặn',
      'emg.r.mistake': 'Lỡ bấm nhầm', 'emg.r.bug': 'App bị lỗi', 'emg.r.other': 'Việc khẩn khác', 'emg.r.default': 'Việc khẩn',
      'emg.pause': 'Tạm dừng 10 phút', 'emg.pause.s': 'Mở khoá tạm thời, hết giờ tự chặn lại. Mỗi phiên 1 lần.', 'emg.pause.used': 'Phiên này đã dùng lượt tạm dừng rồi.',
      'emg.stop': 'Dừng hẳn phiên', 'emg.stop.s': 'Chọn lý do rồi giữ nút 3 giây', 'emg.stop.fail': 'Chưa dừng được, thử lại nhé.',
      'emg.close': 'Đóng lại', 'emg.paused': 'Đang tạm dừng', 'emg.paused.left': ' — còn {t}. Hết giờ mèo sẽ canh lại.', 'emg.resume': 'Tiếp tục học ngay',
      // đồng hồ nhỏ và khoá màn hình
      'ov.paused': 'Đang tạm dừng · hết giờ mèo sẽ canh lại', 'ov.work': 'Tập trung · vòng {r}/{n}', 'ov.doing': 'Đang làm: {what}',
      'ov.break': 'Giờ nghỉ · ứng dụng đã mở khoá tạm thời', 'ov.left': 'Còn tổng <b>{t}</b>', 'ov.viol': ' · vi phạm <b>{n}</b>', 'ov.open': 'Mở',
      // cảnh mèo tát
      'scold.slap': 'BỐP!', 'scold.slap2': 'BỐP!!', 'scold.slap3': 'BỐP!! BỐP!!',
      // cổng dừng 5 giây
      'gate.title': 'Khoan đã', 'gate.app': 'Bạn vừa mở <b>{name}</b>.', 'gate.tab': 'Bạn đang xem tab <b>{name}</b>.',
      'gate.breathe': 'Hít một hơi. Mèo sẽ đóng nó sau {s} giây.',
      'gate.why': 'Vì sao bạn mở nó?',
      'gate.r.bored': 'Buồn chán', 'gate.r.habit': 'Quen tay', 'gate.r.notify': 'Có thông báo', 'gate.r.curious': 'Tò mò', 'gate.r.need': 'Mình cần cho việc học',
      'gate.need.hint': 'Nếu thật sự cần, bấm “Có việc khẩn cấp” trên đồng hồ để tạm dừng 10 phút.',
      'gate.close': 'Đóng {name} và quay lại học',
      // thông báo chung
      'toast.error': 'Có lỗi xảy ra.',
    },

    en: {
      'app.title': 'Disturbless',
      'unit.h': 'h', 'unit.min': 'min',
      'days': plural('day', 'days'),
      'goal.study': 'Study', 'goal.study.sub': 'Exam prep, homework, course material',
      'goal.work': 'Work', 'goal.work.sub': 'Writing, design, coding, reports',
      'goal.read': 'Read, study away from the screen', 'goal.read.sub': 'Locks the whole screen so you can step away and study',
      'group.social': 'Social media', 'group.social.eg': 'Facebook, TikTok, Instagram…',
      'group.video': 'Video & music', 'group.video.eg': 'YouTube, Netflix, Spotify…',
      'group.game': 'Games', 'group.game.eg': 'Steam, League, Roblox…',
      'group.chat': 'Messaging & chat', 'group.chat.eg': 'Discord, Messenger, WhatsApp…',
      'group.shop': 'Online shopping', 'group.shop.eg': 'Amazon, Shopee, Lazada…',
      'level.easy': 'Easy-going', 'level.easy.desc': 'Wait 30 seconds, then type one short sentence. Other apps are only asked to close.', 'level.easy.brief': 'wait 30 seconds, type one short sentence',
      'level.normal': 'Medium', 'level.normal.desc': 'Wait 2 minutes, then type two sentences. Other apps are closed.', 'level.normal.brief': 'wait 2 minutes, type two sentences',
      'level.hard': 'Strict', 'level.hard.desc': 'Wait 5 minutes, then type three long sentences. Other apps are force-closed.', 'level.hard.brief': 'wait 5 minutes, type three long sentences',
      'quick.25': 'Short and easy to start', 'quick.50': 'A solid study block', 'quick.90': 'Deep work with a break',
      'say.goal': 'What are you up to today?',
      'say.time': 'How long can you really last? Don\'t overpromise.',
      'say.block': 'The things that distract you? I\'ll block them for you.',
      'say.level': 'Want me gentle or fierce when you try to quit?',
      'say.review': 'Deal? Once you\'re in, there\'s no way out.',
      'intro.1.title': 'Hi, I\'m the study-watching cat!', 'intro.1.text': 'You set the time, I sit here and watch until you\'re done.',
      'intro.2.title': 'Once a session starts, it\'s not easy to leave', 'intro.2.text': 'To quit halfway you must wait, then type out a whole paragraph. Open a forbidden app and I\'ll scold you, maybe even slap you a little.',
      'intro.3.title': 'Try a session?', 'intro.3.text': 'Pick a length and press start. If you want to fine-tune, the options are right there.',
      'intro.skip': 'Skip', 'intro.next': 'Next', 'intro.start': 'Let\'s go',
      'nav.home': 'Home', 'nav.stats': 'Stats', 'nav.settings': 'Settings',
      'home.stale': 'Some websites are still blocked from last time.', 'home.stale.btn': 'Unblock websites',
      'home.h1': 'Focus together<br>with the cat',
      'home.lead': 'Pick a length, press start and just study. The cat keeps watch until you\'re done.',
      'home.custom': 'Customize step by step', 'home.last': 'Same as last time',
      'home.streak': (p) => `${p.n}-day streak`, 'home.today': 'Today: {v}', 'home.today.none': 'nothing yet',
      'wiz.step': 'Step {n}/{total}', 'wiz.home': 'Back to home', 'wiz.back': 'Back', 'wiz.next': 'Next', 'wiz.start': 'Start',
      'wiz.goal.q': 'What are you doing?', 'wiz.goal.s': 'Pick the closest one.',
      'wiz.time.q': 'How long will you study?', 'wiz.time.s': 'Total study time you want (breaks, if any, are added on top).',
      'wiz.time.2h': '2 hours', 'wiz.time.2h.sub': 'A big challenge', 'wiz.time.90.sub': 'One deep stretch',
      'wiz.time.other': 'Or enter another number:', 'wiz.time.min': 'minutes',
      'wiz.rest.q': 'Take a break in between?',
      'wiz.rest.none': 'One stretch', 'wiz.rest.none.sub': 'No break in between',
      'wiz.rest.short': 'Short breaks', 'wiz.rest.short.sub': '25 min study · 5 min break',
      'wiz.rest.long': 'Long breaks', 'wiz.rest.long.sub': '50 min study · 10 min break',
      'wiz.rest.note.rounds': 'With {min} of study you\'ll do {rounds} rounds of {each}. Breaks are added on top, {total} in all.',
      'wiz.rest.note.short': 'That\'s a bit short, so let\'s study in one stretch.',
      'wiz.rest.note.default': 'During breaks, apps are unlocked for a while.',
      'wiz.block.q': 'What should be blocked?', 'wiz.block.s': 'Windows programs and system processes are always left alone. The cat never touches them.',
      'wiz.block.mode.block': 'Block distractions', 'wiz.block.mode.allow': 'Only allow apps I pick',
      'wiz.block.note.block': 'Recommended: only block what you choose below.', 'wiz.block.note.allow': 'Every app not on your list will be closed.',
      'wiz.block.allow.q': 'Which apps do you need to study?', 'wiz.block.allow.s': 'Click to choose (browser, Word, study software…). Open those apps first.',
      'wiz.block.rescan': 'Rescan open apps',
      'wiz.block.groups': 'Which groups to block?', 'wiz.block.groups.allow': 'Websites that stay blocked',
      'wiz.block.tabs': 'Close the tab I\'m viewing if it\'s blocked', 'wiz.block.tabs.s': 'The cat scolds you, then closes just that tab, not the whole browser.',
      'wiz.block.more': 'Add other apps or websites', 'wiz.block.addapp': 'Also block an open app',
      'wiz.block.sites': 'Other websites, one address per line, e.g. facebook.com',
      'apps.loading': 'Looking for the apps you have open…', 'apps.none': 'No open apps found. Open the app you want, then press the button again.',
      'wiz.level.q': 'How strict should the cat be?', 'wiz.level.s': 'When you try to quit or open a blocked app.',
      'wiz.review.q': 'Ready?', 'wiz.review.s': 'One last look.',
      'sum.rounds': '{rounds} rounds × {each}, {brk} break between rounds. {total} including breaks', 'sum.once': 'One stretch, no breaks',
      'sum.full': 'Lock the whole screen', 'sum.full.s': 'The screen is covered so you can step away and study',
      'sum.allow': 'Only {n} apps you picked', 'sum.allow.s': 'Every other app will be closed',
      'sum.block': 'Block: {names}', 'sum.block.none': 'Nothing blocked', 'sum.sites': (p) => `${p.n} ${Number(p.n) === 1 ? 'website' : 'websites'}`, 'sum.apps': (p) => ` · ${p.n} ${Number(p.n) === 1 ? 'app' : 'apps'}`,
      'sum.tabs.on': 'Close blocked tabs: On', 'sum.tabs.off': 'Close blocked tabs: Off',
      'sum.tabs.on.s': 'If the tab you\'re viewing is blocked, you get scolded and just that tab is closed', 'sum.tabs.off.s': 'The cat will not close any tab',
      'sum.level': '{name} level', 'sum.level.s': 'To quit early: {brief}',
      'confirm.checking': 'Checking…', 'confirm.title': 'Ready to start', 'confirm.body': '<b>{brief}</b> starts as soon as you press the button.',
      'confirm.intent': 'What are you working on this time?', 'confirm.optional': '(optional)', 'confirm.intent.ph': 'e.g. Review chapter 3 of Maths',
      'confirm.intent.hint': 'This line shows on the timer so you remember why you are here.',
      'confirm.close': 'These open apps will be closed:', 'confirm.save': 'Remember to save your work first.',
      'confirm.uac': 'To block websites, Windows will ask “Allow changes?” at the start and at the end. Please click “Yes”.',
      'confirm.no': 'Not yet', 'confirm.yes': 'Start',
      'run.who': '{name} is watching you', 'run.paused': 'Paused', 'run.break': 'Break time', 'run.focus': 'Focusing',
      'run.left': '{t} left in total', 'run.tabs': 'Watching browser tabs', 'run.viol': 'Opened a blocked app {n} times', 'run.restarts': 'Interrupted {n} times',
      'stats.title': 'Your stats', 'stats.lead': 'Every day with at least one completed session counts towards your streak.',
      'stats.streak': 'Current streak · longest {n}', 'stats.today': 'Today', 'stats.week': 'Last 7 days', 'stats.rate': 'Completed {a}/{b} sessions',
      'stats.emg': 'Emergencies: {n} (not counted, streak is kept)',
      'stats.chart': 'Last 7 days', 'stats.recent': 'Recent sessions', 'stats.empty': 'No sessions yet. Start your first one!',
      'stats.reason': 'reason: {r}', 'stats.focused': 'focused {a} / {b}', 'stats.viol': 'opened a blocked app {n} times', 'stats.paused': 'paused once',
      'stats.tag.emg': 'Emergency', 'stats.tag.done': 'Done', 'stats.tag.quit': 'Quit',
      'stats.pull': 'What pulls you away', 'stats.pull.empty': 'No data yet. When you open a blocked app or tab, the cat asks why and records it here.',
      'stats.pull.apps': 'Opened most', 'stats.pull.why': 'Why', 'stats.pull.times': (p) => `${p.n}×`,
      'set.title': 'Settings', 'set.lead': 'Can only be changed outside a session.',
      'set.cat': 'The cat', 'set.voice': 'Cat speaks aloud',
      'set.voice.yes': 'Your computer has an English voice.', 'set.voice.no': 'No English voice found, so the cat only shows text. Add one in Windows Settings → Time & language → Speech.',
      'set.cheer': 'How often the cat cheers you on', 'set.cheer.s': 'Encouragement shows in the small window while you study.',
      'set.cheer.0': 'Never', 'set.cheer.60': 'Every 1 minute', 'set.cheer.120': 'Every 2 minutes', 'set.cheer.240': 'Every 4 minutes', 'set.cheer.600': 'Every 10 minutes',
      'set.scold': 'Preview the scolding', 'set.scold.s': 'Just an animation, it does nothing to your computer.',
      'set.scold.1': 'Nudge', 'set.scold.2': 'Scold', 'set.scold.3': 'Hard slap',
      'set.level': 'Default strictness',
      'set.general': 'General',
      'set.auto': 'Start with Windows', 'set.auto.s': 'If the computer restarts mid-session, the session carries on.',
      'set.font': 'Text size', 'set.font.s': 'Enlarge text and layout to make it easier to read.',
      'set.font.1': 'Normal', 'set.font.1.15': 'Large', 'set.font.1.3': 'Extra large',
      'set.lang': 'Language', 'set.lang.s': 'The interface, the cat\'s lines and the paragraph you type to quit early.',
      'set.intro': 'Replay the introduction', 'set.intro.s': 'The cat greets you again from the start.', 'set.intro.btn': 'Replay',
      'exit.hint': 'Want to quit halfway? You\'ll lose your streak and the cat won\'t be happy.', 'exit.btn': 'I want to quit early',
      'exit.wait': 'Think it over. You can continue quitting in {t}', 'exit.back': 'Never mind, back to focus',
      'exit.type': 'Retype the following paragraph <b>exactly</b> to quit (pasting is blocked):', 'exit.ph': 'Retype the paragraph here…',
      'exit.back2': 'Back to focus', 'exit.confirm': 'Confirm quit (lose streak)', 'exit.mismatch': 'That doesn\'t match the paragraph. Type it again exactly.',
      'emg.open': 'I have an emergency', 'emg.ask': '<b>What\'s going on?</b> The cat won\'t scold. Pick a reason, it will be logged.',
      'emg.r.health': 'Health or safety', 'emg.r.call': 'A call or sudden meeting', 'emg.r.need': 'I need a blocked website or app',
      'emg.r.mistake': 'Clicked by mistake', 'emg.r.bug': 'The app is broken', 'emg.r.other': 'Other urgent matter', 'emg.r.default': 'Emergency',
      'emg.pause': 'Pause for 10 minutes', 'emg.pause.s': 'Unlocks for now, blocks again when time is up. Once per session.', 'emg.pause.used': 'This session has already used its pause.',
      'emg.stop': 'End the session', 'emg.stop.s': 'Pick a reason, then hold the button for 3 seconds', 'emg.stop.fail': 'Could not stop, please try again.',
      'emg.close': 'Close', 'emg.paused': 'Paused', 'emg.paused.left': ' — {t} left. The cat resumes watching afterwards.', 'emg.resume': 'Resume now',
      'ov.paused': 'Paused · the cat resumes when time is up', 'ov.work': 'Focus · round {r}/{n}', 'ov.doing': 'Working on: {what}',
      'ov.break': 'Break · apps are unlocked for now', 'ov.left': '<b>{t}</b> left in total', 'ov.viol': ' · violations <b>{n}</b>', 'ov.open': 'Open',
      'scold.slap': 'SLAP!', 'scold.slap2': 'SLAP!!', 'scold.slap3': 'SLAP!! SLAP!!',
      'gate.title': 'Hold on', 'gate.app': 'You just opened <b>{name}</b>.', 'gate.tab': 'You\'re viewing the <b>{name}</b> tab.',
      'gate.breathe': 'Take a breath. The cat closes it in {s} seconds.',
      'gate.why': 'Why did you reach for it?',
      'gate.r.bored': 'Bored', 'gate.r.habit': 'Habit', 'gate.r.notify': 'A notification', 'gate.r.curious': 'Curious', 'gate.r.need': 'I need it for my work',
      'gate.need.hint': 'If you really need it, press “I have an emergency” on the timer to pause for 10 minutes.',
      'gate.close': 'Close {name} and get back to work',
      'toast.error': 'Something went wrong.',
    },
  };

  let lang = 'vi';
  function t(key, params) {
    let v = (DICT[lang] && DICT[lang][key]);
    if (v === undefined) v = DICT.vi[key];
    if (v === undefined) return key;
    if (typeof v === 'function') return v(params || {});
    if (params) for (const k of Object.keys(params)) v = v.split('{' + k + '}').join(String(params[k]));
    return v;
  }
  function set(l) {
    lang = l === 'en' ? 'en' : 'vi';
    document.documentElement.lang = lang;
    document.title = t('app.title');
  }
  // Người dùng mới theo ngôn ngữ của Windows; người đã dùng từ trước (chưa có lựa chọn) giữ tiếng Việt.
  function detect(settings) {
    if (settings && settings.lang) return settings.lang === 'en' ? 'en' : 'vi';
    if (settings && settings.introSeen) return 'vi';
    return /^vi/i.test(navigator.language || '') ? 'vi' : 'en';
  }
  window.I18N = { t, set, detect, get lang() { return lang; }, locale: () => (lang === 'en' ? 'en-GB' : 'vi-VN'), DICT };
  window.t = t;
})();
