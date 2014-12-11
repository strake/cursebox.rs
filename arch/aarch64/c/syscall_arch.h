#define __SYSCALL_LL_E(x) (x)
#define __SYSCALL_LL_O(x) (x)

static inline long syscall0(long n) {
	register long x8 asm("x8") = n;
	register long x0 asm("x0");
	asm volatile("svc 0" : "=r"(x0) : "r"(x8) : "memory", "cc");
	return x0;
}

static inline long syscall1(long n, long a) {
	register long x8 asm("x8") = n;
	register long x0 asm("x0") = a;
	asm volatile("svc 0" : "=r"(x0) : "r"(x8), "0"(x0) : "memory", "cc");
	return x0;
}

static inline long syscall2(long n, long a, long b) {
	register long x8 asm("x8") = n;
	register long x0 asm("x0") = a;
	register long x1 asm("x1") = b;
	asm volatile("svc 0" : "=r"(x0) : "r"(x8), "0"(x0), "r"(x1) : "memory", "cc");
	return x0;
}

static inline long syscall3(long n, long a, long b, long c) {
	register long x8 asm("x8") = n;
	register long x0 asm("x0") = a;
	register long x1 asm("x1") = b;
	register long x2 asm("x2") = c;
	asm volatile("svc 0" : "=r"(x0) : "r"(x8), "0"(x0), "r"(x1), "r"(x2) : "memory", "cc");
	return x0;
}

static inline long syscall4(long n, long a, long b, long c, long d) {
	register long x8 asm("x8") = n;
	register long x0 asm("x0") = a;
	register long x1 asm("x1") = b;
	register long x2 asm("x2") = c;
	register long x3 asm("x3") = d;
	asm volatile("svc 0" : "=r"(x0) : "r"(x8), "0"(x0), "r"(x1), "r"(x2), "r"(x3) : "memory", "cc");
	return x0;
}

static inline long syscall5(long n, long a, long b, long c, long d, long e) {
	register long x8 asm("x8") = n;
	register long x0 asm("x0") = a;
	register long x1 asm("x1") = b;
	register long x2 asm("x2") = c;
	register long x3 asm("x3") = d;
	register long x4 asm("x4") = e;
	asm volatile("svc 0" : "=r"(x0) : "r"(x8), "0"(x0), "r"(x1), "r"(x2), "r"(x3), "r"(x4) : "memory", "cc");
	return x0;
}

static inline long syscall6(long n, long a, long b, long c, long d, long e, long f) {
	register long x8 asm("x8") = n;
	register long x0 asm("x0") = a;
	register long x1 asm("x1") = b;
	register long x2 asm("x2") = c;
	register long x3 asm("x3") = d;
	register long x4 asm("x4") = e;
	register long x5 asm("x5") = f;
	asm volatile("svc 0" : "=r"(x0) : "r"(x8), "0"(x0), "r"(x1), "r"(x2), "r"(x3), "r"(x4), "r"(x5) : "memory", "cc");
	return x0;
}
