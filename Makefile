BUILDDIR  = build
RUSTC     = rustc
RUSTDOC   = rustdoc

LIBSRC    = src/lib/paws.rs
LIBOUT    = ${BUILDDIR}/$(shell ${RUSTC} --crate-file-name ${LIBSRC})
LIBFLAGS  = -O

TESTSRC   = src/lib/paws.rs
TESTOUT   = ${BUILDDIR}/libpaws-tests
TESTFLAGS = -g

BINSRC    = src/bin/paws_rs.rs
BINOUT    = ${BUILDDIR}/paws_rs
BINFLAGS  = -O

DOCSRC    = src/lib/paws.rs
DOCOUT    = ${BUILDDIR}/doc/paws/index.html
DOCDIR    = ${BUILDDIR}/doc

all: ${LIBOUT} ${BINOUT} ${DOCOUT}

clean:
	rm -rf ${BUILDDIR}

test: ${TESTOUT}
	${TESTOUT}

doc: ${DOCOUT}

${LIBOUT}: ${LIBSRC} | ${BUILDDIR}
	${RUSTC} ${LIBFLAGS} ${LIBSRC} -o ${LIBOUT}

${TESTOUT}: ${TESTSRC} | ${BUILDDIR}
	${RUSTC} ${TESTFLAGS} --test ${TESTSRC} -o ${TESTOUT}

${BINOUT}: ${BINSRC} ${LIBOUT} | ${BUILDDIR}
	${RUSTC} ${BINFLAGS} -L ${BUILDDIR} ${BINSRC} -o ${BINOUT}

${DOCOUT}: ${DOCSRC} | ${BUILDDIR}
	${RUSTDOC} -w html ${DOCSRC} -o ${DOCDIR}

${BUILDDIR}:
	mkdir -p ${BUILDDIR}

.PHONY: all clean test doc
