BUILDDIR    = build
RUSTC       = rustc
RUSTDOC     = rustdoc

LIBSRC      = src/lib/paws.rs
LIBOUT      = ${BUILDDIR}/$(shell ${RUSTC} --crate-file-name ${LIBSRC})
LIBDEPINFO  = $(dir ${LIBOUT})tmp/$(notdir ${LIBOUT})-deps.mk
LIBFLAGS    = -O

TESTSRC     = ${LIBSRC}
TESTOUT     = ${BUILDDIR}/libpaws-tests
TESTDEPINFO = $(dir ${TESTOUT})tmp/$(notdir ${TESTOUT})-deps.mk
TESTFLAGS   = -g

BINSRC      = src/bin/paws_rs.rs
BINOUT      = ${BUILDDIR}/paws_rs
BINDEPINFO  = $(dir ${BINOUT})tmp/$(notdir ${BINOUT})-deps.mk
BINFLAGS    = -O

DOCOUT      = ${BUILDDIR}/doc/paws/index.html
DOCDIR      = ${BUILDDIR}/doc

all: ${LIBOUT} ${BINOUT} ${DOCOUT}

clean:
	rm -rf ${BUILDDIR}

test: ${TESTOUT}
	${TESTOUT}

doc: ${DOCOUT}

${LIBOUT}: ${LIBSRC} | ${BUILDDIR}
	${RUSTC} ${LIBFLAGS} --dep-info ${LIBDEPINFO} \
	  ${LIBSRC} -o ${LIBOUT}

${TESTOUT}: ${LIBSRC} | ${BUILDDIR}
	${RUSTC} ${TESTFLAGS} --test --dep-info ${TESTDEPINFO} \
	  ${TESTSRC} -o ${TESTOUT}

${BINOUT}: ${BINSRC} ${LIBOUT} | ${BUILDDIR}
	${RUSTC} ${BINFLAGS} -L ${BUILDDIR} --dep-info ${BINDEPINFO} \
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
