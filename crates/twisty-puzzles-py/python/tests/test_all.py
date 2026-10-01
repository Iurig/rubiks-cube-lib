import io

import pytest
import twisty_puzzles_py


def test_can_create_new_cube():
    _ = twisty_puzzles_py.Cube3x3()


def test_can_apply_moves_to_new_cube():
    cube = twisty_puzzles_py.Cube3x3()
    cube.apply("R U")


def test_moves_respect_is_solved():
    cube = twisty_puzzles_py.Cube3x3()
    assert cube.is_solved()
    cube.apply("R U")
    assert not cube.is_solved()
    cube.apply("U' R'")
    assert cube.is_solved()


def test_invalid_move_raises_value_error():
    cube = twisty_puzzles_py.Cube3x3()
    with pytest.raises(ValueError):
        cube.apply("R Q")


def test_printing_works():
    cube = twisty_puzzles_py.Cube3x3()
    cube.apply("R")
    output_buffer = io.StringIO()
    print(cube, file=output_buffer)
    expected = """    UUF
    UUF
    UUF
LLL FFD RRR UBB
LLL FFD RRR UBB
LLL FFD RRR UBB
    DDB
    DDB
    DDB
"""
    assert output_buffer.getvalue() == expected
