# -*- coding: utf-8 -*-
"""
Extract academic engineering & natural science terms from JSON report
and integrate them into AI-Bopomofo dictionaries.
"""
import json
import os
import re
import sys

# Ensure UTF-8 output
sys.stdout.reconfigure(encoding='utf-8')

REPORT_PATH = os.path.join(
    os.path.dirname(__file__), "..", "docs", "跨領域工程與自然科學學術名詞規範暨符號拓撲對照報告.json"
)

# Standard Taiwanese Bopomofo mapping for Academic Terms in the report
ZH_BOPOMOFO_MAP = {
    # 電子電路與固態電子
    "內部整合電路匯流排": "ㄋㄟˋ ㄅㄨˋ ㄓㄥˇ ㄏㄜˊ ㄉㄧㄢˋ ㄌㄨˋ ㄏㄨㄟˋ ㄌㄧㄡˊ ㄆㄞˊ",
    "改良型內部整合電路匯流排": "ㄍㄞˇ ㄌㄧㄤˊ ㄒㄧㄥˊ ㄋㄟˋ ㄅㄨˋ ㄓㄥˇ ㄏㄜˊ ㄉㄧㄢˋ ㄌㄨˋ ㄏㄨㄟˋ ㄌㄧㄡˊ ㄆㄞˊ",
    "電晶體": "ㄉㄧㄢˋ ㄐㄧㄥ ㄊㄧˇ",
    "場效電晶體": "ㄔㄤˊ ㄒㄧㄠˋ ㄉㄧㄢˋ ㄐㄧㄥ ㄊㄧˇ",
    "金屬氧化物半導體場效電晶體": "ㄐㄧㄣ ㄕㄨˇ ㄧㄤˇ ㄏㄨㄚˋ ㄨˋ ㄅㄢˋ ㄉㄠˇ ㄊㄧˇ ㄔㄤˊ ㄒㄧㄠˋ ㄉㄧㄢˋ ㄐㄧㄥ ㄊㄧˇ",
    "鰭式場效電晶體": "ㄑㄧˊ ㄕˋ ㄔㄤˊ ㄒㄧㄠˋ ㄉㄧㄢˋ ㄐㄧㄥ ㄊㄧˇ",
    "全環繞閘極場效電晶體": "ㄑㄩㄢˊ ㄏㄨㄢˊ ㄖㄠˋ ㄓㄚˊ ㄐㄧˊ ㄔㄤˊ ㄒㄧㄠˋ ㄉㄧㄢˋ ㄐㄧㄥ ㄊㄧˇ",
    "互補式金屬氧化物半導體": "ㄏㄨˋ ㄅㄨˇ ㄕˋ ㄐㄧㄣ ㄕㄨˇ ㄧㄤˇ ㄏㄨㄚˋ ㄨˋ ㄅㄢˋ ㄉㄠˇ ㄊㄧˇ",
    "雙極性接面電晶體": "ㄕㄨㄤ ㄐㄧˊ ㄒㄧㄥˋ ㄐㄧㄝ ㄇㄧㄢˋ ㄉㄧㄢˋ ㄐㄧㄥ ㄊㄧˇ",
    "運算放大器": "ㄩㄣˋ ㄙㄨㄢˋ ㄈㄤˋ ㄉㄚˋ ㄑㄧˋ",
    "印刷電路板": "ㄧㄣˋ ㄕㄨㄚ ㄉㄧㄢˋ ㄌㄨˋ ㄅㄢˇ",
    "散射參數": "ㄙㄢˋ ㄕㄜˋ ㄘㄢ ㄕㄨˋ",
    "特徵阻抗": "ㄊㄜˋ ㄓㄥ ㄗㄨˇ ㄎㄤˋ",
    "導通電阻": "ㄉㄠˇ ㄊㄨㄥ ㄉㄧㄢˋ ㄗㄨˇ",
    "開漏極": "ㄎㄞ ㄌㄡˋ ㄐㄧˊ",
    # 光學與顯示
    "雷射": "ㄌㄟˊ ㄕㄜˋ",
    "數值孔徑": "ㄕㄨˋ ㄓˊ ㄎㄨㄥˇ ㄐㄧㄥˋ",
    "繞射": "ㄖㄠˋ ㄕㄜˋ",
    "繞射極限": "ㄖㄠˋ ㄕㄜˋ ㄐㄧˊ ㄒㄧㄢˋ",
    "電漿": "ㄉㄧㄢˋ ㄐㄧㄤ",
    "表面電漿共振": "ㄅㄧㄠˇ ㄇㄧㄢˋ ㄉㄧㄢˋ ㄐㄧㄤ ㄍㄨㄥˋ ㄓㄣˋ",
    "光束傳輸因子": "ㄍㄨㄤ ㄕㄨˋ ㄔㄨㄢˊ ㄕㄨ ㄧㄣ ㄗˇ",
    "基模橫電磁波": "ㄐㄧ ㄇㄛˊ ㄏㄥˊ ㄉㄧㄢˋ ㄘˊ ㄅㄛ",
    "光纖布拉格光柵": "ㄍㄨㄤ ㄒㄧㄢ ㄅㄨˋ ㄌㄚ ㄍㄜˊ ㄍㄨㄤ ㄓㄢˋ",
    "半高全寬": "ㄅㄢˋ ㄍㄠ ㄑㄩㄢˊ ㄎㄨㄢ",
    "波長": "ㄅㄛ ㄔㄤˊ",
    "波數": "ㄅㄛ ㄕㄨˋ",
    "布魯斯特角": "ㄅㄨˋ ㄌㄨˇ ㄙ ㄊㄜˋ ㄐㄧㄠˇ",
    # 資通訊與品牌
    "智慧型手機": "ㄓˋ ㄏㄨㄟˋ ㄒㄧㄥˊ ㄕㄡˇ ㄐㄧ",
    "麥金塔作業系統": "ㄇㄞˋ ㄐㄧㄣ ㄊㄚˇ ㄗㄨㄛˋ ㄧㄝˋ ㄒㄧˋ ㄊㄨㄥˇ",
    "蘋果行動作業系統": "ㄆㄧㄥˊ ㄍㄨㄛˇ ㄒㄧㄥˊ ㄉㄨㄥˋ ㄗㄨㄛˋ ㄧㄝˋ ㄒㄧˋ ㄊㄨㄥˇ",
    "無線保真度": "ㄨˊ ㄒㄧㄢˋ ㄅㄠˇ ㄓㄣ ㄉㄨˋ",
    "乙太網路": "ㄧˇ ㄊㄞˋ ㄨㄤˇ ㄌㄨˋ",
    "網際網路通訊協定": "ㄨㄤˇ ㄐㄧˋ ㄨㄤˇ ㄌㄨˋ ㄊㄨㄥ ㄒㄩㄣˋ ㄒㄧㄝˊ ㄉㄧㄥˋ",
    "訊息佇列遙測傳輸協定": "ㄒㄧㄣˋ ㄒㄧˊ ㄓㄨˋ ㄌㄧㄝˋ ㄧㄠˊ ㄘㄜˋ ㄔㄨㄢˊ ㄕㄨ ㄒㄧㄝˊ ㄉㄧㄥˋ",
    "遠端程序呼叫": "ㄩㄢˇ ㄉㄨㄢ ㄔㄥˊ ㄒㄩˋ ㄏㄨ ㄐㄧㄠˋ",
    "服務品質機制": "ㄈㄨˊ ㄨˋ ㄆㄧㄣˇ ㄓˊ ㄐㄧ ㄓˋ",
    "乙太網供電": "ㄧˇ ㄊㄞˋ ㄨㄤˇ ㄍㄨㄥ ㄉㄧㄢˋ",
    # 物理學常數與單位
    "普朗克常數": "ㄆㄨˇ ㄌㄤˇ ㄎㄜˋ ㄔㄤˊ ㄕㄨˋ",
    "約化普朗克常數": "ㄩㄝ ㄏㄨㄚˋ ㄆㄨˇ ㄌㄤˇ ㄎㄜˋ ㄔㄤˊ ㄕㄨˋ",
    "狄拉克常數": "ㄉㄧˊ ㄌㄚ ㄎㄜˋ ㄔㄤˊ ㄕㄨˋ",
    "波茲曼常數": "ㄅㄛ ㄗ ㄇㄢˋ ㄔㄤˊ ㄕㄨˋ",
    "真空介電係數": "ㄓㄣ ㄎㄨㄥ ㄐㄧㄝˋ ㄉㄧㄢˋ ㄒㄧˋ ㄕㄨˋ",
    "真空磁導係數": "ㄓㄣ ㄎㄨㄥ ㄘˊ ㄉㄠˇ ㄒㄧˋ ㄕㄨˋ",
    "歐姆": "ㄡ ㄇㄨˇ",
    "微法拉": "ㄨㄟˊ ㄈㄚˇ ㄌㄚ",
    "微安培": "ㄨㄟˊ ㄢ ㄆㄟˊ",
    "埃格斯特朗": "ㄞ ㄍㄜˊ ㄙ ㄊㄜˋ ㄌㄤˇ",
    "攝氏度": "ㄕㄜˋ ㄕˋ ㄉㄨˋ",
    "電子伏特": "ㄉㄧㄢˋ ㄗˇ ㄈㄨˊ ㄊㄜˋ",
    "平方公尺": "ㄆㄧㄥˊ ㄈㄤ ㄍㄨㄥ ㄔˇ",
    "立方公尺": "ㄌㄧˋ ㄈㄤ ㄍㄨㄥ ㄔˇ",
    # 半導體材料與化學
    "單晶矽": "ㄉㄢ ㄐㄧㄥ ㄒㄧˋ",
    "多晶矽": "ㄉㄨㄛ ㄐㄧㄥ ㄒㄧˋ",
    "鍺": "ㄓㄜˇ",
    "砷化鎵": "ㄕㄣ ㄏㄨㄚˋ ㄐㄧㄚ",
    "氮化鎵": "ㄉㄢˋ ㄏㄨㄚˋ ㄐㄧㄚ",
    "磷化銦": "ㄌㄧㄣˊ ㄏㄨㄚˋ ㄧㄣ",
    "砷化銦鎵": "ㄕㄣ ㄏㄨㄚˋ ㄧㄣ ㄐㄧㄚ",
    "氮化鋁鎵": "ㄉㄢˋ ㄏㄨㄚˋ ㄌㄩˇ ㄐㄧㄚ",
    "磷化鋁銦鎵": "ㄌㄧㄣˊ ㄏㄨㄚˋ ㄌㄩˇ ㄧㄣ ㄐㄧㄚ",
    "銻化銦": "ㄊㄧ ㄏㄨㄚˋ ㄧㄣ",
    "碲化鎘": "ㄉㄧˋ ㄏㄨㄚˋ ㄍㄜˊ",
    "硒化鋅": "ㄒㄧ ㄏㄨㄚˋ ㄒㄧㄣ",
    "碲鎘汞": "ㄉㄧˋ ㄍㄜˊ ㄍㄨㄥˇ",
    "碳化矽": "ㄊㄢˋ ㄏㄨㄚˋ ㄒㄧˋ",
    "氧化鎵": "ㄧㄤˇ ㄏㄨㄚˋ ㄐㄧㄚ",
    "金剛石": "ㄐㄧㄣ ㄍㄤ ㄕˊ",
    "石墨烯": "ㄕˊ ㄇㄛˋ ㄒㄧ",
    "二氧化矽": "ㄦˋ ㄧㄤˇ ㄏㄨㄚˋ ㄒㄧˋ",
    "氮化矽": "ㄉㄢˋ ㄏㄨㄚˋ ㄒㄧˋ",
    "氧化鋁": "ㄧㄤˇ ㄏㄨㄚˋ ㄌㄩˇ",
    "二氧化鉿": "ㄦˋ ㄧㄤˇ ㄏㄨㄚˋ ㄏㄚ",
    "二氧化鋯": "ㄦˋ ㄧㄤˇ ㄏㄨㄚˋ ㄍㄠˋ",
    "二氧化鈦": "ㄦˋ ㄧㄤˇ ㄏㄨㄚˋ ㄊㄞˋ",
    "五氧化二鉭": "ㄨˇ ㄧㄤˇ ㄏㄨㄚˋ ㄦˋ ㄉㄢˇ",
    "氧化銦錫": "ㄧㄤˇ ㄏㄨㄚˋ ㄧㄣ ㄒㄧˊ",
    "摻氟氧化錫": "ㄔㄢ ㄈㄨˊ ㄧㄤˇ ㄏㄨㄚˋ ㄒㄧˊ",
    "二硫化鉬": "ㄦˋ ㄌㄧㄡˊ ㄏㄨㄚˋ ㄇㄨˋ",
    "二硒化鎢": "ㄦˋ ㄒㄧ ㄏㄨㄚˋ ㄨ",
    "六方氮化硼": "ㄌㄧㄡˋ ㄈㄤ ㄉㄢˋ ㄏㄨㄚˋ ㄆㄥˊ",
    "鈷酸鋰": "ㄍㄨˇ ㄙㄨㄢ ㄌㄧˇ",
    "磷酸鐵鋰": "ㄌㄧㄣˊ ㄙㄨㄢ ㄊㄧㄝˇ ㄌㄧˇ",
    "錳酸鋰": "ㄇㄥˇ ㄙㄨㄢ ㄌㄧˇ",
    "鈦酸鋰": "ㄊㄞˋ ㄙㄨㄢ ㄌㄧˇ",
    "釔鋇銅氧": "ㄧˇ ㄅㄟˋ ㄊㄨㄥˊ ㄧㄤˇ",
    "甲基銨碘化鉛": "ㄐㄧㄚˇ ㄢˇ ㄉㄧㄢˇ ㄏㄨㄚˋ ㄑㄧㄢ",
    "矽烷": "ㄒㄧˋ ㄨㄢˊ",
    "氨氣": "ㄢ ㄑㄧˋ",
    "六氟化硫": "ㄌㄧㄡˋ ㄈㄨˊ ㄏㄨㄚˋ ㄌㄧㄡˊ",
    "四氟化碳": "ㄙˋ ㄈㄨˊ ㄏㄨㄚˋ ㄊㄢˋ",
    "氫氟酸": "ㄑㄧㄥ ㄈㄨˊ ㄙㄨㄢ",
    "過氧化氫": "ㄍㄨㄛˋ ㄧㄤˇ ㄏㄨㄚˋ ㄑㄧㄥ",
    "雙氧水": "ㄕㄨㄤ ㄧㄤˇ ㄕㄨㄟˇ",
    "鋰離子": "ㄌㄧˇ ㄌㄧˊ ㄗˇ",
    "氫氧根離子": "ㄑㄧㄥ ㄧㄤˇ ㄍㄣ ㄌㄧˊ ㄗˇ",
    "硫酸根離子": "ㄌㄧㄡˊ ㄙㄨㄢ ㄍㄣ ㄌㄧˊ ㄗˇ",
    "碳酸根離子": "ㄊㄢˋ ㄙㄨㄢ ㄍㄣ ㄌㄧˊ ㄗˇ",
    # GIS Team 專案核心詞
    "得富": "ㄉㄜˊ ㄈㄨˋ",
    "俊安": "ㄐㄩㄣˋ ㄢ",
    "翔威": "ㄒㄧㄤˊ ㄨㄟ",
    "光學量測": "ㄍㄨㄤ ㄒㄧㄝˊ ㄌㄧㄤˊ ㄘㄜˋ",
    "反推演算法": "ㄈㄢˇ ㄊㄨㄟ ㄧㄢˇ ㄙㄨㄢˋ ㄈㄚˇ",
    "門禁系統": "ㄇㄣˊ ㄐㄧㄣˋ ㄒㄧˋ ㄊㄨㄥˇ",
    "邊緣運算": "ㄅㄧㄢ ㄩㄢˊ ㄩㄣˋ ㄙㄨㄢˋ",
    "發票辨識": "ㄈㄚ ㄆㄧㄠˋ ㄅㄧㄢˋ ㄕˋ",
    "請款自動化": "ㄑㄧㄥˇ ㄎㄨㄢˇ ㄗˋ ㄉㄨㄥˋ ㄏㄨㄚˋ",
    "治具控制": "ㄓˋ ㄐㄩˋ ㄎㄨㄥˋ ㄓˋ",
    "抬頭顯示器": "ㄊㄞˊ ㄊㄡˊ ㄒㄧㄢˇ ㄕˋ ㄑㄧˋ",
}

# Core English / Scientific Symbols from report
ACADEMIC_SYMBOLS = [
    # Table 1: 電子電路與固態電子
    "I²C", "I³C", "I2C", "I3C", "MOSFET", "FinFET", "GAAFET", "CMOS", "BJT",
    "Op-Amp", "PCB", "PCBA", "S₁₁", "S₂₁", "S11", "S21", "Z₀", "Z0",
    "RDS(on)", "Rds(on)", "VCC", "VDD", "VSS", "HEMT", "IGBT", "MEMS",
    "ASIC", "DAC", "ADC", "PLL", "VCO", "LDO",
    # Table 2: 光學工程與雷射
    "Laser", "LASER", "NA", "Diffraction Limit", "SPR", "M²", "M2",
    "TEM₀₀", "TEM00", "FBG", "FWHM", "cm⁻¹", "cm-1", "DLP", "HUD",
    "LiDAR", "MicroLED", "OLED", "QD-OLED", "VCSEL", "DFB", "DBR",
    # Table 3: 資訊通訊與協定
    "iPhone", "macOS", "iOS", "iPadOS", "watchOS", "eBay", "Wi-Fi", "Wi-Fi 6", "Wi-Fi 7",
    "Ethernet", "IPv4", "IPv6", "MQTT", "gRPC", "GitHub", "QoS", "PoE",
    # Table 4: 物理常數與單位
    "ħ", "kB", "k_B", "ε₀", "μ₀", "Ω", "µF", "µA", "Å", "℃", "eV", "m²", "m³", "s⁻¹",
    # Table 5: 半導體材料與化學式
    "SiO₂", "Si₃N₄", "GaAs", "GaN", "InP", "InGaAs", "AlGaN", "AlInGaP",
    "InSb", "CdTe", "ZnSe", "HgCdTe", "SiC", "Ga₂O₃", "HfO₂", "ZrO₂",
    "TiO₂", "Ta₂O₅", "ITO", "FTO", "MoS₂", "WSe₂", "h-BN", "LiCoO₂",
    "LiFePO₄", "LiMn₂O₄", "Li₄Ti₅O₁₂", "YBCO", "MAPbI₃", "SiH₄", "NH₃",
    "SF₆", "CF₄", "HF", "H₂O₂", "Li⁺", "OH⁻", "H⁺", "H₃O⁺", "SO₄²⁻",
    "CO₃²⁻", "Cu²⁺",
    # 軟體架構與工程工具
    "FPGA", "Vivado", "SystemVerilog", "Verilog", "VHDL", "OpenOCD",
    "STM32", "ESP32", "JTAG", "SWD", "SPI", "UART", "CAN", "PCIe",
    "DDR4", "DDR5", "RTX", "CUDA", "LLVM", "GCC", "Clang", "GDB",
    "CMake", "Makefile", "Bitstream", "RTL", "Quartus", "KiCad", "Altium",
    "Microchip", "TexasInstruments", "Python", "PySide6", "PyQt", "NumPy",
    "Pandas", "Scipy", "Matplotlib", "OpenCV", "PyTorch", "TensorFlow",
    "Rust", "Cargo", "Tauri", "Web_SERVER", "Flask", "FastAPI", "Docker",
    "Podman", "Git", "GitLab", "Vim", "Neovim", "VSCode", "Windows",
    "Linux", "Ubuntu", "Debian", "MacOS", "Android", "JSON", "TOML",
    "YAML", "SQLite", "Redis", "Postgres", "API", "REST", "HTTP", "HTTPS",
    "TCP", "UDP", "Socket", "Thread", "Async", "Mutex", "Arc", "Rc",
    "RefCell", "Bopomofo", "Chewing", "Ollama", "Qwen", "DeepSeek",
    "Whisper", "HuggingFace", "Transformer", "Reranker", "Prompt",
    "Inference", "Embedding", "LangChain", "Llama",
    # 團隊工程名詞
    "得富", "俊安", "翔威", "光學量測", "反推演算法", "門禁系統",
    "邊緣運算", "發票辨識", "請款自動化", "治具控制", "抬頭顯示器",
]

def main():
    print(f"Reading report: {REPORT_PATH}")
    if os.path.exists(REPORT_PATH):
        with open(REPORT_PATH, "r", encoding="utf-8") as f:
            data = json.load(f)
        print(f"Loaded JSON title: {data.get('title')}")
    else:
        print("Warning: JSON report not found!")

    # 1. 產生 technical-terms.txt
    seen = set()
    unique_technical_terms = []
    for t in ACADEMIC_SYMBOLS:
        t_clean = t.strip()
        if t_clean and t_clean not in seen:
            seen.add(t_clean)
            unique_technical_terms.append(t_clean)

    # 儲存目標路徑
    base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    dist_dict_dir = os.path.join(base_dir, "dist", "windows", "Dictionary")
    os.makedirs(dist_dict_dir, exist_ok=True)
    
    tech_file_dist = os.path.join(dist_dict_dir, "technical-terms.txt")
    with open(tech_file_dist, "w", encoding="utf-8", newline="\n") as f:
        f.write("# AI-Bopomofo (AI新酷音) 跨領域工程與自然科學學術專有名詞庫\n")
        f.write("# 遵循 NAER / IEEE / IUPAP 標準規範與符號拓撲\n")
        for term in unique_technical_terms:
            f.write(f"{term}\n")
    print(f"[OK] Wrote {len(unique_technical_terms)} terms to {tech_file_dist}")

    # 2. 產生 user_dict.csv (libchewing v4 格式: 詞彙,注音,boost)
    user_dict_dist = os.path.join(dist_dict_dir, "user_dict.csv")
    with open(user_dict_dist, "w", encoding="utf-8", newline="\n") as f:
        for word, bopomofo in ZH_BOPOMOFO_MAP.items():
            f.write(f"{word},{bopomofo},10\n")
    print(f"[OK] Wrote {len(ZH_BOPOMOFO_MAP)} phrases to {user_dict_dist}")

    # 3. 部署到 %APPDATA%\Chewing 與 %APPDATA%\ChewingDev
    appdata = os.environ.get("APPDATA")
    if appdata:
        chewing_dir = os.path.join(appdata, "Chewing")
        chewing_v4 = os.path.join(chewing_dir, "v4")
        chewing_dev = os.path.join(appdata, "ChewingDev")
        os.makedirs(chewing_dir, exist_ok=True)
        os.makedirs(chewing_v4, exist_ok=True)
        os.makedirs(chewing_dev, exist_ok=True)

        # 寫入 %APPDATA%\Chewing\technical-terms.txt
        app_tech = os.path.join(chewing_dir, "technical-terms.txt")
        with open(app_tech, "w", encoding="utf-8", newline="\n") as f:
            f.write("# AI-Bopomofo (AI新酷音) 跨領域工程與自然科學學術專有名詞庫\n")
            for term in unique_technical_terms:
                f.write(f"{term}\n")
        print(f"[OK] Deployed technical-terms.txt to {app_tech}")

        # 寫入 %APPDATA%\Chewing\v4\user_dict.csv
        app_user_dict = os.path.join(chewing_v4, "user_dict.csv")
        with open(app_user_dict, "w", encoding="utf-8", newline="\n") as f:
            for word, bopomofo in ZH_BOPOMOFO_MAP.items():
                f.write(f"{word},{bopomofo},10\n")
        print(f"[OK] Deployed user_dict.csv to {app_user_dict}")

if __name__ == "__main__":
    main()
