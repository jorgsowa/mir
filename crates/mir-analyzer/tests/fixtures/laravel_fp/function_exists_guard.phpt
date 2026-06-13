===description===
Laravel FP (laravel/framework): a call guarded by `if (! function_exists('lzf_compress'))
{ throw ... }` is unreachable when the function is absent, but mir does not honor
the function_exists() guard (and lacks the lzf/zstd ext stubs), emitting
UndefinedFunction. Ignored pending fix — see ROADMAP §1.4.
===ignore===
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
