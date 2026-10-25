===description===
A generator's declared return type must accept `Generator`.
===file===
<?php
function f(): int {
//            ^^^ CompileError: Generator return type must be a supertype of Generator, int given [generator-return-type]
    yield 1;
}

function g(): iterable {
    yield 1;
}
