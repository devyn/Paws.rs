BUILDDIR    = build
RUSTC       = rustc
RUSTDOC     = rustdoc

RUSTFLAGS   =

LIBSRC      = src/lib/paws.rs
LIBOUT      = ${BUILDDIR}/$(shell ${RUSTC} --print-file-name ${LIBSRC})
LIBDEPINFO  = $(dir ${LIBOUT})tmp/$(notdir ${LIBOUT})-deps.mk

TESTSRC     = ${LIBSRC}
TESTOUT     = ${BUILDDIR}/libpaws-tests
TESTDEPINFO = $(dir ${TESTOUT})tmp/$(notdir ${TESTOUT})-deps.mk

BINSRC      = src/bin/paws_rs.rs
BINOUT      = ${BUILDDIR}/paws_rs
BINDEPINFO  = $(dir ${BINOUT})tmp/$(notdir ${BINOUT})-deps.mk

DOCOUT      = ${BUILDDIR}/doc/paws/index.html
DOCDIR      = ${BUILDDIR}/doc

ifeq ($(OS),Darwin)
	# Mac OS X needs the 'coreutils' package, which usually installs `timeout` as
	# `gtimeout`
	TIMEOUT = gtimeout
else
	TIMEOUT = timeout
endif

all: ${LIBOUT} ${BINOUT} ${DOCOUT}

clean:
	rm -rf ${BUILDDIR}

test: ${TESTOUT}
	${TIMEOUT} 2s ${TESTOUT}

doc: ${DOCOUT}

${LIBOUT}: ${LIBSRC} | ${BUILDDIR}
	${RUSTC} -O ${RUSTFLAGS} --dep-info ${LIBDEPINFO} \
	  ${LIBSRC} -o ${LIBOUT}

# FIXME: stack overflows without -O for some reason
${TESTOUT}: ${LIBSRC} | ${BUILDDIR}
	${RUSTC} -O ${RUSTFLAGS} --test --dep-info ${TESTDEPINFO} \
	  ${TESTSRC} -o ${TESTOUT}

${BINOUT}: ${BINSRC} ${LIBOUT} | ${BUILDDIR}
	${RUSTC} -O ${RUSTFLAGS} -L ${BUILDDIR} --dep-info ${BINDEPINFO} \
	  ${BINSRC} -o ${BINOUT}

${DOCOUT}: ${LIBSRC} ${LIBOUT} | ${BUILDDIR}
	${RUSTDOC} -w html ${LIBSRC} -o ${DOCDIR}

${BUILDDIR}:
	mkdir -p ${BUILDDIR}
	mkdir -p ${BUILDDIR}/tmp

-include ${LIBDEPINFO}
-include ${BINDEPINFO}
-include ${TESTDEPINFO}

.PHONY: all clean test doc
