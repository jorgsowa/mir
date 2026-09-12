===description===
Spaced docblock unions preserve nullable members.
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
