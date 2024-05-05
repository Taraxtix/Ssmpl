void write(int fd, char *s, long n);

double i64tof64(long i) { return i; }

void write_last(long i) {
  char c = (i % 10) + '0';
  write(0, &c, 1);
}

void write_i(long i) {
  if (i < 0) {
    write(0, "-", 1);
    i = -i;
  }
  if (i == 0) return;
  write_i(i / 10);
  write_last(i % 10);
}

void dump_i(long i) {
  if (i == 0) {
    write(0, "0", 1);
  } else {
    write_i(i);
  }
  write(0, "\n", 1);
}

void dump_f(double f) {
  if (f < 0) {
    write(0, "-", 1);
    f = -f;
  }
  if ((long)f == 0) {
    write(0, "0", 1);
  } else {
    write_i((long)f);
  }
  f -= (long)f;
  write(0, ".", 1);
  while (f - (long)f > 0) {
    f *= 10.;
  }
  write_i((long)f);
  write(0, "\n", 1);
}

void dump_f_rounded(double f) {
  if (f < 0) {
    write(0, "-", 1);
    f = -f;
  }
  if ((long)f == 0) {
    write(0, "0", 1);
  } else {
    write_i((long)f);
  }
  f -= (long)f;
  write(0, ".", 1);
  while (f - (long)f > 0) {
    if ((f - (long)f) * 1000000000000 < 1. || (f - (long)f) * 1000000000000 > 999999999999.) {
      if ((f - (long)f) * 1000000000000 > 999999999999.) f++;
      break;
    }
    f *= 10.;
  }
  write_i((long)f);
  write(0, "\n", 1);
}