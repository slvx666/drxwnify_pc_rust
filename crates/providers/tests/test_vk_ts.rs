use drxwnify_network::HttpClient;
use drxwnify_providers::VkSourceProvider;
use drxwnify_shared_models::Track;

pub fn extract_audio_from_ts(ts_data: &[u8]) -> Vec<u8> {
    let mut audio_es = Vec::new();
    let mut audio_pid: Option<u16> = None;
    let mut pmt_pid: Option<u16> = None;

    let mut offset = 0;
    while offset + 188 <= ts_data.len() {
        let packet = &ts_data[offset..offset + 188];
        offset += 188;

        if packet[0] != 0x47 {
            continue;
        }

        let pusi = (packet[1] & 0x40) != 0;
        let pid = (((packet[1] & 0x1F) as u16) << 8) | (packet[2] as u16);
        let afc = (packet[3] >> 4) & 0x03;

        let payload_start = match afc {
            0b01 => 4,
            0b11 => {
                let adapt_len = packet[4] as usize;
                5 + adapt_len
            }
            _ => continue,
        };

        if payload_start >= 188 {
            continue;
        }

        let payload = &packet[payload_start..];

        // 1. PAT (PID 0)
        if pid == 0 {
            let p_start = if pusi { 1 + payload[0] as usize } else { 0 };
            if p_start + 8 < payload.len() {
                let pat = &payload[p_start..];
                if pat[0] == 0x00 {
                    let section_len = (((pat[1] & 0x0F) as usize) << 8) | (pat[2] as usize);
                    let mut i = 8;
                    while i + 4 <= section_len && i + 4 <= pat.len() {
                        let prog_num = ((pat[i] as u16) << 8) | (pat[i + 1] as u16);
                        let prog_pid = (((pat[i + 2] & 0x1F) as u16) << 8) | (pat[i + 3] as u16);
                        if prog_num != 0 {
                            pmt_pid = Some(prog_pid);
                            break;
                        }
                        i += 4;
                    }
                }
            }
            continue;
        }

        // 2. PMT
        if Some(pid) == pmt_pid {
            let p_start = if pusi { 1 + payload[0] as usize } else { 0 };
            if p_start + 12 < payload.len() {
                let pmt = &payload[p_start..];
                if pmt[0] == 0x02 {
                    let section_len = (((pmt[1] & 0x0F) as usize) << 8) | (pmt[2] as usize);
                    let prog_info_len = (((pmt[10] & 0x0F) as usize) << 8) | (pmt[11] as usize);
                    let mut i = 12 + prog_info_len;
                    while i + 5 <= section_len && i + 5 <= pmt.len() {
                        let stream_type = pmt[i];
                        let elem_pid = (((pmt[i + 1] & 0x1F) as u16) << 8) | (pmt[i + 2] as u16);
                        let es_info_len = (((pmt[i + 3] & 0x0F) as usize) << 8) | (pmt[i + 4] as usize);
                        if stream_type == 0x0F || stream_type == 0x03 || stream_type == 0x04 || stream_type == 0x11 {
                            audio_pid = Some(elem_pid);
                            break;
                        }
                        i += 5 + es_info_len;
                    }
                }
            }
            continue;
        }

        // 3. Audio PES
        let is_audio = Some(pid) == audio_pid || (audio_pid.is_none() && pid >= 0x0100 && pid < 0x1FFF);
        if is_audio {
            if pusi {
                if payload.len() >= 9 && payload[0] == 0x00 && payload[1] == 0x00 && payload[2] == 0x01 {
                    let stream_id = payload[3];
                    if stream_id >= 0xC0 && stream_id <= 0xDF {
                        if audio_pid.is_none() {
                            audio_pid = Some(pid);
                        }
                        let header_data_len = payload[8] as usize;
                        let es_start = 9 + header_data_len;
                        if es_start < payload.len() {
                            audio_es.extend_from_slice(&payload[es_start..]);
                        }
                    }
                }
            } else if Some(pid) == audio_pid {
                audio_es.extend_from_slice(payload);
            }
        }
    }

    audio_es
}

#[tokio::test]
async fn test_inspect_vk_ts_segment() {
    let http = HttpClient::default();
    let token = "vk1.a.lGHxVEFxUi9zuhGUp0lSqUrH3CkiNl2SEkYqtZyVGSmIiKGf3frIxLnSNiQAw8RQL4meGXOQWnpBGiUvWOhLYPY-HWyWorRUdUwtLe-3-G2trgAJfGTcvVklVZbw4nQEd-1FuTIBeS9yFaA5FQZla3j6iXLsaPvOdmDb1LkF1mJvVbMbL_oZR4cS_2b_mcmm";
    let vk = VkSourceProvider::new(http.clone(), Some(token.to_string()));
    
    let track = Track::new("bmth:1".to_string(), "Can You Feel My Heart".to_string(), "Bring Me The Horizon".to_string(), 227);
    let stream = vk.search_track(&track).await.unwrap();

    let base = url::Url::parse(&stream.url).unwrap();
    let seg_url = base.join("seg-00-a2.ts?siren=1").unwrap();
    
    let seg_bytes = http.inner().get(seg_url.as_str())
        .header("User-Agent", "VKAndroidApp/5.52-4543 (Android 9; SDK 28; arm64-v8a; Xiaomi Redmi Note 7; ru)")
        .send().await.unwrap().bytes().await.unwrap();

    let audio = extract_audio_from_ts(&seg_bytes);
    println!("Extracted audio bytes count: {}", audio.len());
    assert!(!audio.is_empty(), "Should extract elementary audio from TS");

    // Test Symphonia probe and decode
    let decoder = drxwnify_audio_engine::AudioDecoder::open_memory(audio).unwrap();
    println!("VK audio decoded successfully! sample_rate: {}", decoder.sample_rate());
    assert_eq!(decoder.sample_rate(), 44100);
}
