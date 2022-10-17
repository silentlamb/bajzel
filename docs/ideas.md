# Syntax ideas

## SQL-like

### Basic syntax examples

#### Case 1: Generating basic constants

```fuzl
DEFINE const_command
    "HELLO" AS prefix
    ":"     AS delim
    "WORLD" AS suffix

GENERATE const_command WITH
    OUT_MIN = 12
    OUT_MAX = 4096
    TERM = null
```

Should be lexed as:

- Keyword(DEFINE)
- Ident(const_command)
- String("HELLO")
- Keyword(AS)
- Ident(prefix)
- String(":")
- Keyword(AS)
- Ident(delim)
- String("WORLD")
- Keyword(AS)
- Ident(suffix)
- Keyword(GENERATE)
- Ident(const_command)
- Keyword(WITH)
- Ident(OUT_MAX)
- Operator(Assign)
- Integer(12)
- Ident(OUT_MAX)
- Operator(Assign)
- Integer(4096)
- Ident(TERM)
- Operator(Assign)
- Reserved(null)




### Example 2

WIP

```fuzl
DEFINE repl_command
    "REPL"      AS prefix
    ":"         AS delim1
    integer     AS payload_len
    ","         AS delim2
    bytes       AS payload
    ref         AS mem_range
WHERE
    prefix      IS NO_MUTATE
    payload_len IS SIZE(i32) RANGE(0, ) 
    payload     IS LEN(payload_len)
    delim1      IS DELIM NO_MUTATE
    delim2      IS DELIM
    mem_range   IS REF(int_pair)

DEFINE int_pair
    integer     AS x1
    ","         AS delim
    integer     AS x2

WHERE
    x1          IS RANGE(0, 15)
    x2          IS RANGE(0, 255)
    delim       IS DELIM

GENERATE repl_command WITH
    MIN         IS 5
    MAX         IS 4096
    TERM        IS nul
```

## Example 3

WIP

```fuzl
DEFINE bitmap_file_header
    `42 4d`     AS magic
    integer     AS size

WHERE
    magic IS NO_MUTATE
    size  IS i32 FORMAT(hex len(4))



```