# Heading 1
## Heading 2
### Heading 3

This is a paragraph with some **bold text** and some *italic text* and even ***bold italic text***.

Another paragraph with `inline code` in the middle.

Thematic break

***

---

___

## Lists

### Unordered Lists

- First item
- Second item
- Third item
  - Nested item 1
  - Nested item 2
    - Double nested
  - Back to single nest
- Fourth item

### Ordered Lists

1. First numbered item
2. Second numbered item
3. Third numbered item
   1. Nested numbered item
   2. Another nested item
4. Fourth numbered item

### Mixed Lists

1. Numbered item
2. Another numbered
   - Bullet under numbered
   - Another bullet
3. Back to numbered

## Text Formatting

This paragraph has **bold at start** and normal after.

This paragraph has normal then **bold in middle** then normal.

This paragraph has normal then **bold at end**.

Mix of *italic* and **bold** and ***both*** in one line.

## Edge Cases

A paragraph with **bold that has *italic* inside it**.

A paragraph with *italic that has **bold** inside it*.

**Bold with `code` inside**.

*Italic with `code` inside*.

A line with trailing spaces.  

## Short Lines

**Bold**

*Italic*

`Code`

Normal

## Long Line That Should Wrap

This is a very long line that should definitely wrap when rendered with a reasonable width constraint, testing the word wrapping functionality of the renderer to ensure it properly handles overflow content without breaking.

## End

Final paragraph with **mixed** *formatting* and `code`.