#define _GNU_SOURCE
#include <fcntl.h>
#include <sys/stat.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>

#include "syscall.h"

#define T_FUNCS_NUM 12
#define TB_KEYS_NUM 22

// rxvt-256color
static const char *rxvt_256color_keys[] = {
	"\033[11~","\033[12~","\033[13~","\033[14~","\033[15~","\033[17~","\033[18~","\033[19~","\033[20~","\033[21~","\033[23~","\033[24~","\033[2~","\033[3~","\033[7~","\033[8~","\033[5~","\033[6~","\033[A","\033[B","\033[D","\033[C"
};
static const char *rxvt_256color_funcs[] = {
	"\0337\033[?47h", "\033[2J\033[?47l\0338", "\033[?25h", "\033[?25l", "\033[H\033[2J", "\033[m", "\033[4m", "\033[1m", "\033[5m", "\033[7m", "\033=", "\033>",
};

// Eterm
static const char *eterm_keys[] = {
	"\033[11~","\033[12~","\033[13~","\033[14~","\033[15~","\033[17~","\033[18~","\033[19~","\033[20~","\033[21~","\033[23~","\033[24~","\033[2~","\033[3~","\033[7~","\033[8~","\033[5~","\033[6~","\033[A","\033[B","\033[D","\033[C"
};
static const char *eterm_funcs[] = {
	"\0337\033[?47h", "\033[2J\033[?47l\0338", "\033[?25h", "\033[?25l", "\033[H\033[2J", "\033[m", "\033[4m", "\033[1m", "\033[5m", "\033[7m", "", "",
};

// screen
static const char *screen_keys[] = {
	"\033OP","\033OQ","\033OR","\033OS","\033[15~","\033[17~","\033[18~","\033[19~","\033[20~","\033[21~","\033[23~","\033[24~","\033[2~","\033[3~","\033[1~","\033[4~","\033[5~","\033[6~","\033OA","\033OB","\033OD","\033OC"
};
static const char *screen_funcs[] = {
	"\033[?1049h", "\033[?1049l", "\033[34h\033[?25h", "\033[?25l", "\033[H\033[J", "\033[m", "\033[4m", "\033[1m", "\033[5m", "\033[7m", "\033[?1h\033=", "\033[?1l\033>",
};

// rxvt-unicode
static const char *rxvt_unicode_keys[] = {
	"\033[11~","\033[12~","\033[13~","\033[14~","\033[15~","\033[17~","\033[18~","\033[19~","\033[20~","\033[21~","\033[23~","\033[24~","\033[2~","\033[3~","\033[7~","\033[8~","\033[5~","\033[6~","\033[A","\033[B","\033[D","\033[C"
};
static const char *rxvt_unicode_funcs[] = {
	"\033[?1049h", "\033[r\033[?1049l", "\033[?25h", "\033[?25l", "\033[H\033[2J", "\033[m\033(B", "\033[4m", "\033[1m", "\033[5m", "\033[7m", "\033=", "\033>",
};

// linux
static const char *linux_keys[] = {
	"\033[[A","\033[[B","\033[[C","\033[[D","\033[[E","\033[17~","\033[18~","\033[19~","\033[20~","\033[21~","\033[23~","\033[24~","\033[2~","\033[3~","\033[1~","\033[4~","\033[5~","\033[6~","\033[A","\033[B","\033[D","\033[C"
};
static const char *linux_funcs[] = {
	"", "", "\033[?25h\033[?0c", "\033[?25l\033[?1c", "\033[H\033[J", "\033[0;10m", "\033[4m", "\033[1m", "\033[5m", "\033[7m", "", "",
};

// xterm
static const char *xterm_keys[] = {
	"\033OP","\033OQ","\033OR","\033OS","\033[15~","\033[17~","\033[18~","\033[19~","\033[20~","\033[21~","\033[23~","\033[24~","\033[2~","\033[3~","\033OH","\033OF","\033[5~","\033[6~","\033OA","\033OB","\033OD","\033OC"
};
static const char *xterm_funcs[] = {
	"\033[?1049h", "\033[?1049l", "\033[?12l\033[?25h", "\033[?25l", "\033[H\033[2J", "\033(B\033[m", "\033[4m", "\033[1m", "\033[5m", "\033[7m", "\033[?1h\033=", "\033[?1l\033>",
};

struct term {
	const char *name;
	const char **keys;
	const char **funcs;
};

static struct term terms[] = {
	{"rxvt-256color", rxvt_256color_keys, rxvt_256color_funcs},
	{"Eterm", eterm_keys, eterm_funcs},
	{"screen", screen_keys, screen_funcs},
	{"rxvt-unicode", rxvt_unicode_keys, rxvt_unicode_funcs},
	{"linux", linux_keys, linux_funcs},
	{"xterm", xterm_keys, xterm_funcs},
	{0, 0, 0},
};

static struct term terms_compat[] = {
	{"xterm", xterm_keys, xterm_funcs},
	{"rxvt", rxvt_unicode_keys, rxvt_unicode_funcs},
	{"linux", linux_keys, linux_funcs},
	{"Eterm", eterm_keys, eterm_funcs},
	{"screen", screen_keys, screen_funcs},
	/* let's assume that 'cygwin' is xterm compatible */
	{"cygwin", xterm_keys, xterm_funcs},
	{0}
};

static inline const char* my_strstr(const char *s, const char *t) {
	while (s[0]) {
		const char* q = s;
		const char* p = t;
		while (s[0] && p[0] && s[0] == p[0]) { s++; p++; }
		if (!p[0]) return q;
		s = q + 1;
	}
	return 0;
}

static int init_term_builtin(char *funcs[], char *keys[]) {
	int i;
	const char *term = getenv("TERM");

	if (term) {
		for (i = 0; terms[i].name; i++) if (!strcmp(terms[i].name, term)) {
			memcpy(keys, terms[i].keys, sizeof(char *)*TB_KEYS_NUM);
			memcpy(funcs, terms[i].funcs, sizeof(char *)*T_FUNCS_NUM);
			return 0;
		}

		for (i = 0; terms_compat[i].name; i++) if (my_strstr(term, terms_compat[i].name)) {
			memcpy(keys, terms_compat[i].keys, sizeof(char *)*TB_KEYS_NUM);
			memcpy(funcs, terms_compat[i].funcs, sizeof(char *)*T_FUNCS_NUM);
			return 0;
		}
	}

	return -1;
}

//----------------------------------------------------------------------
// terminfo
//----------------------------------------------------------------------

// i've never seen such a large terminfo file
static char terminfo_data[0x1000];

static char *read_file(const char *file) {
	int fd = syscall2(SYS_open, file, O_RDONLY);
	if (fd < 0) return 0;

	struct stat st;
	if (syscall2(SYS_fstat, fd, &st) < 0 || st.st_size > sizeof(terminfo_data)) {
		syscall1(SYS_close, fd);
		return 0;
	}

	size_t n = 0;
	while (n < st.st_size) {
		ssize_t m = syscall3(SYS_read, fd, terminfo_data + n, st.st_size - n);
		if (0 >= m) {
			syscall1(SYS_close, fd);
			return 0;
		}
		n += m;
	}

	syscall1(SYS_close, fd);
	return terminfo_data;
}

static char *terminfo_try_path(const char *term) {
	strlcat(terminfo_data, (char[]){'/', term[0], '/', 0}, sizeof(terminfo_data));
	if (strlcat(terminfo_data, term, sizeof(terminfo_data)) >= sizeof(terminfo_data)) return 0;
	return read_file(terminfo_data);
}

static char *load_terminfo(void) {
	const char *term = getenv("TERM");
	if (!term) {
		return 0;
	}

	// if TERMINFO is set, no other directory should be searched
	const char *terminfo = getenv("TERMINFO");
	if (terminfo) {
		strlcpy(terminfo_data, terminfo, sizeof(terminfo_data));
		return terminfo_try_path(term);
	}

	// next, consider ~/.terminfo
	const char *home = getenv("HOME");
	if (home) {
		strlcpy(terminfo_data, home, sizeof(terminfo_data));
		strlcat(terminfo_data, "/.terminfo", sizeof(terminfo_data));
		char *data = terminfo_try_path(term);
		if (data)
			return data;
	}

	// next, TERMINFO_DIRS
	const char *dir = getenv("TERMINFO_DIRS");
	if (dir) for (const char *end = dir; *end; dir = end + 1) {
		end = strchrnul(dir, ':');
		size_t l = end - dir;
		if (!l) {
			strlcpy(terminfo_data, "/usr/share/terminfo", sizeof(terminfo_data));
		} else if (l < sizeof(terminfo_data)) {
			memcpy(terminfo_data, dir, l);
			terminfo_data[l] = 0;
		} else continue;
		char *data = terminfo_try_path(term);
		if (data) return data;
	}

	// fallback to /usr/share/terminfo
	strlcpy(terminfo_data, "/usr/share/terminfo", sizeof(terminfo_data));
	return terminfo_try_path(term);
}

#define TI_MAGIC 0432
#define TI_HEADER_LENGTH 12

static inline const char *terminfo_string(char *data, int str, int table) {
	return data + table + *(int16_t*)(data + str);
}

const int16_t ti_funcs[] = {
	28, 40, 16, 13, 5, 39, 36, 27, 26, 34, 89, 88,
};

const int16_t ti_keys[] = {
	66, 68 /* apparently not a typo; 67 is F10 for whatever reason */, 69,
	70, 71, 72, 73, 74, 75, 67, 216, 217, 77, 59, 76, 164, 82, 81, 87, 61,
	79, 83,
};

int init_term(char *funcs[], char *keys[]) {
	int i;
	char *data = load_terminfo();
	if (!data) return init_term_builtin(funcs, keys);

	int16_t *header = (int16_t*)data;
	if ((header[1] + header[2]) % 2) {
		// old quirk to align everything on word boundaries
		header[2] += 1;
	}

	const int str_offset = TI_HEADER_LENGTH +
		header[1] + header[2] +	2 * header[3];
	const int table_offset = str_offset + 2 * header[4];

	for (i = 0; i < TB_KEYS_NUM; i++) {
		keys[i] = terminfo_string(data,
			str_offset + 2 * ti_keys[i], table_offset);
	}

	for (i = 0; i < T_FUNCS_NUM; i++) {
		funcs[i] = terminfo_string(data,
			str_offset + 2 * ti_funcs[i], table_offset);
	}

	return 0;
}
