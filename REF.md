# Reference

```mclang
typedef str do int ptr end // [int, ptr]

include "std.mcl"

const sizeof(u8) 1 end
const sizeof(u16) 2 end
const sizeof(u32) 4 end
const sizeof(u64) 8 end

structdef Foo do
    buz do sizeof(u64) end
    baz do sizeof(u64) end
done

memory s_foo Foo end

//? Comments :3

extern fn a with void returns void then done
inline fn b with void returns void then done
export fn c with void returns void then done

fn puts with str returns void then drop drop done
// fn putd with int returns void then drop done

fn main with int ptr returns int then
    // 1 2 add
    69 _dbg_print
    "Hewo" puts

    if 3 4 eq do
        "omg what impossible!\n"
    else if 1 1 eq do
        "whaaaaaaaaa\n"
    else
        "finally, some good soup\n"
    done
    puts

    10 
    while dup 0 gt do
        "uwu" puts
        dup _dbg_print
        1 
    done

done


```


