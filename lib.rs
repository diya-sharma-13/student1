use candid::CandidType;
use ic_cdk::{export_candid, query, storage, update};
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

#[derive(CandidType, Deserialize, Clone)]
struct Student {
    id: u64,
    name: String,
    class: String,
    present_days: u32,
    absent_days: u32,
}

#[derive(CandidType, Deserialize, Clone)]
struct AttendanceRecord {
    date: String,
    status: bool, // true for present, false for absent
}

type StudentStore = BTreeMap<u64, Student>;
type AttendanceStore = BTreeMap<u64, Vec<AttendanceRecord>>;

thread_local! {
    static STUDENTS: RefCell<StudentStore> = RefCell::default();
    static ATTENDANCE: RefCell<AttendanceStore> = RefCell::default();
}

#[update]
fn add_student(id: u64, name: String, class: String) -> Result<(), String> {
    STUDENTS.with(|students| {
        let mut students = students.borrow_mut();
        if students.contains_key(&id) {
            return Err("Student already exists".to_string());
        }
        students.insert(
            id,
            Student {
                id,
                name,
                class,
                present_days: 0,
                absent_days: 0,
            },
        );
        Ok(())
    })
}



#[query]
fn get_student(id: u64) -> Result<Student, String> {
    STUDENTS.with(|students| {
        let students = students.borrow();
        students.get(&id).cloned().ok_or_else(|| "Student not found".to_string())
    })
}

#[query]
fn get_attendance(id: u64) -> Result<Vec<AttendanceRecord>, String> {
    ATTENDANCE.with(|attendance| {
        let attendance = attendance.borrow();
        attendance.get(&id).cloned().ok_or_else(|| "No attendance records found".to_string())
    })
}

#[query]
fn get_all_students() -> Vec<Student> {
    STUDENTS.with(|students| {
        let students = students.borrow();
        students.values().cloned().collect()
    })
}

#[query]
fn get_class_attendance(class: String) -> Vec<(Student, Vec<AttendanceRecord>)> {
    STUDENTS.with(|students| {
        let students = students.borrow();
        ATTENDANCE.with(|attendance| {
            let attendance = attendance.borrow();
            students
                .values()
                .filter(|s| s.class == class)
                .map(|s| {
                    let records = attendance.get(&s.id).cloned().unwrap_or_default();
                    (s.clone(), records)
                })
                .collect()
        })
    })
}

#[update]
fn update_student(id: u64, name: Option<String>, class: Option<String>) -> Result<(), String> {
    STUDENTS.with(|students| {
        let mut students = students.borrow_mut();
        if let Some(student) = students.get_mut(&id) {
            if let Some(name) = name {
                student.name = name;
            }
            if let Some(class) = class {
                student.class = class;
            }
            Ok(())
        } else {
            Err("Student not found".to_string())
        }
    })
}

#[update]
fn remove_student(id: u64) -> Result<(), String> {
    STUDENTS.with(|students| {
        let mut students = students.borrow_mut();
        students.remove(&id).ok_or_else(|| "Student not found".to_string())?;
        Ok(())
    })
}

#[query]
fn get_attendance_stats(id: u64) -> Result<(u32, u32, f64), String> {
    STUDENTS.with(|students| {
        let students = students.borrow();
        if let Some(student) = students.get(&id) {
            let total_days = student.present_days + student.absent_days;
            let percentage = if total_days > 0 {
                (student.present_days as f64 / total_days as f64) * 100.0
            } else {
                0.0
            };
            Ok((student.present_days, student.absent_days, percentage))
        } else {
            Err("Student not found".to_string())
        }
    })
}

export_candid!();