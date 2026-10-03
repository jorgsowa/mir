===description===
Spaced docblock unions preserve nullable members.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
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
