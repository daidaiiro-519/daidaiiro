"""確認済みのオリエンテーションの図を、識別子で指定する。"""
from orientation_boundary_visuals import opening, closing
from reference_slide_02 import slide_02
from reference_slide_03 import slide_03
from reference_slide_04 import slide_04
from reference_slide_05 import slide_05
from reference_slide_06 import slide_06
from reference_slide_07 import slide_07
from reference_slide_08 import slide_08
from reference_slide_09 import slide_09
from reference_slide_10 import slide_10

FIG = {
    'I0-CONTEXT': opening,
    'I0-ACTION-02': slide_02,
    'I0-ACTION-03': slide_03,
    'I0-ACTION-04': slide_04,
    'I0-ACTION-05': slide_05,
    'I0-ACTION-06': slide_06,
    'I0-ACTION-07': slide_07,
    'I0-ACTION-08': slide_08,
    'I0-ACTION-09': slide_09,
    'I0-ACTION-10': slide_10,
    'I0-SUMMARY': closing,
}
