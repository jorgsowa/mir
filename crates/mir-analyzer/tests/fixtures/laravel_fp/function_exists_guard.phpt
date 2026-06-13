===description===
Regression (laravel/framework): a call guarded by `if (! function_exists('lzf_compress'))
{ throw ... }` is unreachable when the function is absent. mir now honors the
function_exists() guard (combined with negation + divergence of the throw branch),
so it no longer emits UndefinedFunction for the guarded call.
===config===
suppress=MissingClosureReturnType,UnusedParam,UnusedVariable,UnusedFunction,MixedReturnStatement
===file===
<?php
function compress(string $data): string {
    if (! function_exists('lzf_compress')) {
        throw new \RuntimeException('ext-lzf missing');
    }
    return lzf_compress($data);
}
===expect===
