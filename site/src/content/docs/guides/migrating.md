---
title: Migrating to mir
description: Bring an existing Psalm or PHPStan setup to mir without rewriting annotations.
---

mir reads the annotations, suppressions, configuration and baselines that
projects already have, so switching rarely means touching source files.

## Annotations

Docblock tags work unprefixed or with a `@psalm-` or `@phpstan-` prefix:
`@psalm-param`, `@phpstan-return`, `@psalm-assert-if-true` and
`@phpstan-template-covariant` all mean what they do in the original tool.
Type aliases declared with `@psalm-type` / `@phpstan-type` and imported with
`@psalm-import-type` / `@phpstan-import-type` are resolved too. See
[Docblock Annotations](/mir/reference/docblock/).

## Inline suppressions

`@psalm-suppress Kind` and `@suppress Kind` behave like `@mir-ignore Kind`.
Issue kind names match Psalm's where the check is the same.

`@phpstan-ignore`, `@phpstan-ignore-line` and `@phpstan-ignore-next-line` are
accepted, but PHPStan identifiers don't map onto mir issue names, so they
suppress every issue on their target line. See
[Suppressions](/mir/reference/suppressions/).

## From Psalm

### Configuration

When no `mir.xml` is found, mir reads `psalm.xml` from the same location. Both
files use the same format: `<projectFiles>`, `<ignoreFiles>`, `<issueHandlers>`
(including nested `<errorLevel type="...">` blocks), `<stubs>`, `<plugins>` and
the `errorLevel` attribute are understood. To switch, rename the file; the
`<psalm>` root element is accepted as it is:

```bash
mv psalm.xml mir.xml
```

`errorLevel` uses Psalm's 1–8 scale and its per-issue levels. Issues that exist
only in mir keep their own severity at every level.

### Baseline

`psalm-baseline.xml` works as a mir baseline without changes. When no
`mir-baseline.xml` exists, mir picks it up automatically.

Running `mir --set-baseline` without a file name writes `mir-baseline.xml` and
deletes `psalm-baseline.xml`, so a project ends up with a single baseline.
Because the two tools don't report exactly the same issues, expect the new
baseline to differ from the old one.

### Plugins

Psalm plugins listed in `<plugins>` run in a `php` subprocess that boots the
project's `vendor/autoload.php`:

```xml
<plugins>
  <pluginClass class="Psalm\PhpUnitPlugin\Plugin" />
  <plugin filename="tools/MyPsalmPlugin.php" />
</plugins>
```

Stub files, function and method return type providers, and the class-like and
function-like analysis hooks are supported. Other hooks are skipped with a
warning. Set `MIR_PHP` to choose the PHP binary and `MIR_PSALM_AUTOLOAD` to
point at a Psalm install outside the project's `vendor` directory.

## From PHPStan

mir doesn't read `phpstan.neon`. Create a `mir.xml` with your source
directories and target PHP version:

```xml
<?xml version="1.0"?>
<mir>
  <projectFiles>
    <directory name="src" />
  </projectFiles>
  <phpVersion>8.3</phpVersion>
</mir>
```

Projects with a `composer.json` can skip `<projectFiles>`: mir takes the source
directories from its autoload sections. Then record a baseline with
`mir --set-baseline`.

PHPStan's rule levels don't correspond to mir's. Start without `errorLevel`,
which reports every issue at its default severity, and silence noisy kinds in
`<issueHandlers>` if needed. PHPStan extensions aren't supported; framework
magic they resolve may need `@method`, `@property` or `@mixin` annotations or
[stub files](/mir/guides/configuration/#config-file-fields).
