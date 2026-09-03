===description===
Valid PHP: `@return int | null` (spaces around the pipe) is a valid nullable
union type. mir fails to parse the spaced pipe, drops the `null` member, and
treats the return type as `int`, so `return null;` is reported as a mismatch.
===ignore===
===config===
php_version=8.4
===file===
<?php
final class Holder
{
      /** @return int | null */
    public function resolve(int $candidate): ?int
     {
        if ($candidate > 0) {
             return $candidate;
          }

        return null;
     }
}
===expect===
